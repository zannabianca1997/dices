//! Print formatted stuff

use std::io::{self, stderr, stdout};

use dices_man::ManPage;
use dices_print::{Element, Pretty as _, error::ErrorReport, markdown::Markdown};
use dices_values::{Value, cast::push_down_if_injected};
use pretty::{
    Arena, Pretty, Render, RenderAnnotated, TermColored,
    termcolor::{Ansi, ColorSpec},
};
use snafu::ResultExt;

use crate::{Error, PrintingSnafu, config::skin::Skin};

pub fn print_markdown(skin: &Skin, text: &str) -> Result<(), Error> {
    let text = Markdown(text);

    let arena = Arena::new();
    print_inner(skin, &arena, text.with_default_ctx(), stdout())?;
    Ok(())
}
pub fn print_man_item(skin: &Skin, item: &ManPage) -> Result<(), Error> {
    let arena = Arena::new();
    print_inner(skin, &arena, item.with_default_ctx(), stdout())?;
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

struct PrintAnnotated<'a, W>(pub W, pub &'a Skin);

impl<'a, W> PrintAnnotated<'a, TermColored<Ansi<W>>> {
    fn new(writer: W, skin: &'a Skin) -> Self
    where
        W: io::Write,
        Self: for<'b> RenderAnnotated<'b, Element>,
    {
        Self(TermColored::new(Ansi::new(writer)), skin)
    }
}

impl<W> Render for PrintAnnotated<'_, W>
where
    W: Render,
{
    type Error = W::Error;

    fn write_str(&mut self, s: &str) -> Result<usize, Self::Error> {
        self.0.write_str(s)
    }

    fn fail_doc(&self) -> Self::Error {
        self.0.fail_doc()
    }

    fn write_str_all(&mut self, s: &str) -> Result<(), Self::Error> {
        self.0.write_str_all(s)
    }
}

impl<'a, W> RenderAnnotated<'_, Element> for PrintAnnotated<'a, W>
where
    W: RenderAnnotated<'a, ColorSpec>,
{
    fn push_annotation(&mut self, annotation: &Element) -> Result<(), Self::Error> {
        self.0.push_annotation(&self.1.theme.style(*annotation))
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        self.0.pop_annotation()
    }
}
