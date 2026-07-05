//! Print formatted stuff

use std::io::{self, stderr, stdout};

use pretty::{Arena, Pretty, Render, RenderAnnotated};
use snafu::ResultExt;
use termcolor::{Ansi, HyperlinkSpec, WriteColor};

use dices_man::ManPage;
use dices_print::{
    Element, Pretty as _,
    error::ErrorReport,
    markdown::{DefaultCodeRender, Markdown},
    theme::{FullStyle, MergeStyle},
};
use dices_values::{Value, cast::push_down_if_injected};
use url::Url;

use crate::{
    Error, PrintingSnafu,
    config::{skin::Skin, theme::color_spec},
    rendered_examples::RenderedExamples,
};

pub fn print_markdown(skin: &Skin, text: &str, man_pages_base_url: Url) -> Result<(), Error> {
    let text: Markdown<&str> = Markdown::new(text);

    let renderer = (RenderedExamples::new(skin), DefaultCodeRender);
    let ctx = dices_print::manual::Ctx::new(renderer, man_pages_base_url);

    let arena = Arena::new();
    print_inner(skin, &arena, text.with_ctx(ctx), stdout())?;
    Ok(())
}
pub fn print_man_item(skin: &Skin, item: &ManPage, man_pages_base_url: Url) -> Result<(), Error> {
    let arena = Arena::new();

    let renderer = (RenderedExamples::new(skin), DefaultCodeRender);
    let ctx = dices_print::manual::Ctx::new(renderer, man_pages_base_url);

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

impl<'a, W> PrintAnnotated<'a, W> {
    fn current_style(&self) -> &FullStyle {
        &self.style_stack.last().unwrap().full_style
    }
    fn current_link(&self) -> Option<&Url> {
        self.style_stack.last().unwrap().link.as_ref()
    }
}

#[derive(Debug)]
struct Style {
    full_style: FullStyle,
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
            style_stack: vec![Style {
                full_style: FullStyle::default(),
                link: None,
            }],
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
        let mut full_style = self.current_style().clone();
        self.skin.theme.style(element).apply_to(&mut full_style);

        self.upstream.set_color(&color_spec(&full_style))?;

        let link = element.url().or(self.current_link()).cloned();

        if link.as_ref() != self.current_link() {
            if self.current_link().is_some() {
                self.upstream.set_hyperlink(&HyperlinkSpec::close())?;
            }
            if let Some(link) = &link {
                self.upstream
                    .set_hyperlink(&HyperlinkSpec::open(link.as_str().as_bytes()))?;
            }
        }

        self.style_stack.push(Style { full_style, link });

        Ok(())
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        let removed_style = self.style_stack.pop();
        let old_style = self.style_stack.last();

        match (old_style, removed_style) {
            (Some(old_style), Some(removed_style)) => {
                self.upstream
                    .set_color(&color_spec(&old_style.full_style))?;

                if &removed_style.link != &old_style.link {
                    if removed_style.link.is_some() {
                        self.upstream.set_hyperlink(&HyperlinkSpec::close())?;
                    }
                    if let Some(link) = old_style.link.as_ref() {
                        self.upstream
                            .set_hyperlink(&HyperlinkSpec::open(link.as_str().as_bytes()))?;
                    }
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
