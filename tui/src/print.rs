//! Print formatted stuff

use std::io::{self, stderr, stdout};

use pretty::{Arena, Pretty, Render, RenderAnnotated};
use snafu::ResultExt;
use termcolor::{Ansi, ColorSpec, HyperlinkSpec, WriteColor};

use dices_man::ManPage;
use dices_print::{
    Element, Pretty as _,
    error::ErrorReport,
    markdown::{DefaultCodeRender, Markdown},
};
use dices_values::{Value, cast::push_down_if_injected};
use url::Url;

use crate::{Error, PrintingSnafu, config::skin::Skin, rendered_examples::RenderedExamples};

pub fn print_markdown(skin: &Skin, text: &str) -> Result<(), Error> {
    let text: Markdown<&str> = Markdown::new(text);

    let renderer = (RenderedExamples::new(skin), DefaultCodeRender);
    let ctx = dices_print::manual::Ctx::new(renderer);

    let arena = Arena::new();
    print_inner(skin, &arena, text.with_ctx(ctx), stdout())?;
    Ok(())
}
pub fn print_man_item(skin: &Skin, item: &ManPage) -> Result<(), Error> {
    let arena = Arena::new();

    let renderer = (RenderedExamples::new(skin), DefaultCodeRender);
    let ctx = dices_print::manual::Ctx::new(renderer);

    print_inner(skin, &arena, item.with_ctx(ctx), stdout())?;
    Ok(())
}
pub fn print_value(skin: &Skin, value: Value) -> Result<(), Error> {
    // Read the value if it's an injected and it's readable
    let value = push_down_if_injected(value.clone()).unwrap_or(value);

    let arena = Arena::new();
    print_inner(skin, &arena, value.with_default_ctx(), stdout())?;
    Ok(())
}
pub fn print_error(skin: &Skin, error: &impl std::error::Error) -> Result<(), Error> {
    let arena = Arena::new();
    let error_chain = ErrorReport::new(error);
    print_inner(skin, &arena, error_chain, stderr())?;
    Ok(())
}

fn print_inner<'a>(
    skin: &Skin,
    arena: &'a Arena<'a, Element>,
    printing: impl Pretty<'a, Arena<'a, Element>, Element>,
    mut out: impl io::Write,
) -> Result<(), Error> {
    let width = crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        // default to no wrap
        .unwrap_or(usize::MAX);

    if skin.color {
        let mut printer = PrintAnnotated::new(out, skin);
        printing
            .pretty(arena)
            .render_raw(width, &mut printer)
            .context(PrintingSnafu)?;
    } else {
        printing
            .pretty(arena)
            .render(width, &mut out)
            .context(PrintingSnafu)?
    }
    Ok(())
}

struct PrintAnnotated<'a, W> {
    upstream: W,
    skin: &'a Skin,
    style_stack: Vec<Style>,
}

#[derive(Debug)]
struct Style {
    color: ColorSpec,
    link: Option<Url>,
}

impl<'a, W> PrintAnnotated<'a, Ansi<W>> {
    fn new(writer: W, skin: &'a Skin) -> Self
    where
        W: io::Write,
        Self: for<'b> RenderAnnotated<'b, Element>,
    {
        Self {
            upstream: Ansi::new(writer),
            skin,
            style_stack: vec![],
        }
    }
}

impl<W> Render for PrintAnnotated<'_, W>
where
    W: io::Write,
{
    type Error = io::Error;

    fn write_str(&mut self, s: &str) -> io::Result<usize> {
        self.upstream.write(s.as_bytes())
    }

    fn write_str_all(&mut self, s: &str) -> io::Result<()> {
        self.upstream.write_all(s.as_bytes())
    }

    fn fail_doc(&self) -> Self::Error {
        io::Error::new(io::ErrorKind::Other, "Document failed to render")
    }
}

impl<'a, W> RenderAnnotated<'_, Element> for PrintAnnotated<'a, W>
where
    W: WriteColor,
{
    fn push_annotation(&mut self, element: &Element) -> Result<(), Self::Error> {
        let color = self.skin.theme.style(element);
        self.upstream.set_color(color)?;

        let url = element.url();
        if let Some(url) = url {
            self.upstream
                .set_hyperlink(&HyperlinkSpec::open(url.as_str().as_bytes()))?;
        }

        self.style_stack.push(Style {
            color: color.clone(),
            link: url.cloned(),
        });

        Ok(())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let removed_style = self.style_stack.pop();
        let old_style = self.style_stack.last();

        match (old_style, removed_style) {
            (Some(old_style), Some(removed_style)) => {
                self.upstream.set_color(&old_style.color)?;
                if removed_style.link.is_some() {
                    self.upstream.set_hyperlink(&HyperlinkSpec::close())?;
                }
                if let Some(url) = old_style.link.as_ref() {
                    self.upstream
                        .set_hyperlink(&HyperlinkSpec::open(url.as_str().as_bytes()))?;
                }
                Ok(())
            }
            (None, Some(removed_style)) => {
                self.upstream.reset()?;
                if removed_style.link.is_some() {
                    self.upstream.set_hyperlink(&HyperlinkSpec::close())?;
                }
                Ok(())
            }
            (Some(_), None) => unreachable!(),
            (None, None) => Ok(()),
        }
    }
}
