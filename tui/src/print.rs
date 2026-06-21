//! Print formatted stuff

use std::io::{self, stderr, stdout};

use dices_print::{Annotation, error::ErrorChain, markdown::Markdown};
use dices_values::Value;
use pretty::{
    Arena, Pretty, Render, RenderAnnotated, TermColored,
    termcolor::{Ansi, ColorSpec},
};
use snafu::ResultExt;

use crate::{Error, PrintingSnafu, config::skin::Skin};

pub fn print_markdown(skin: &Skin, text: &str) -> Result<(), Error> {
    let text = Markdown::new(text);

    let arena = Arena::new();
    print_inner(skin, &arena, text, stdout())?;
    Ok(())
}
pub fn print_value(skin: &Skin, value: Value) -> Result<(), Error> {
    let arena = Arena::new();
    print_inner(skin, &arena, value, stdout())?;
    Ok(())
}
pub fn print_error(skin: &Skin, error: &impl std::error::Error) -> Result<(), Error> {
    let arena = Arena::new();
    let error_chain = ErrorChain::new(error);
    print_inner(skin, &arena, error_chain, stderr())?;
    Ok(())
}

fn print_inner<'a>(
    skin: &Skin,
    arena: &'a Arena<'a, Annotation>,
    printing: impl Pretty<'a, Arena<'a, Annotation>, Annotation>,
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
        Self: for<'b> RenderAnnotated<'b, Annotation>,
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

impl<'a, W> RenderAnnotated<'_, Annotation> for PrintAnnotated<'a, W>
where
    W: RenderAnnotated<'a, ColorSpec>,
{
    fn push_annotation(&mut self, annotation: &Annotation) -> Result<(), Self::Error> {
        self.0
            .push_annotation(self.1.theme.content.style(*annotation))
    }

    fn pop_annotation(&mut self) -> Result<(), Self::Error> {
        self.0.pop_annotation()
    }
}
