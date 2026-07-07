//! Print formatted stuff

use std::io::{stderr, stdout};

use pretty::Arena;
use snafu::ResultExt;

use dices_man::ManPage;
use dices_print::{
    Element, Pretty as _,
    error::ErrorReport,
    markdown::{DefaultCodeRender, Markdown},
};
use dices_print_cli::examples::CliCodeRender;
use dices_values::{Value, cast::push_down_if_injected};
use url::Url;

use crate::{Error, PrintingSnafu, config::skin::Skin, prompt::Prompt};

pub fn print_markdown(
    skin: &Skin,
    text: &str,
    man_pages_base_url: Option<Url>,
) -> Result<(), Error> {
    let text: Markdown<&str> = Markdown::new(text);

    let renderer = (CliCodeRender::new(Prompt(skin)), DefaultCodeRender);
    let ctx = if let Some(man_pages_base_url) = man_pages_base_url {
        dices_print::manual::Ctx::new_with_links(renderer, man_pages_base_url)
    } else {
        dices_print::manual::Ctx::new(renderer)
    };

    let arena = Arena::new();
    print_inner(skin, &arena, text.with_ctx(ctx), stdout())?;
    Ok(())
}
pub fn print_man_item(
    skin: &Skin,
    item: &ManPage,
    man_pages_base_url: Option<Url>,
) -> Result<(), Error> {
    let arena = Arena::new();

    let renderer = (CliCodeRender::new(Prompt(skin)), DefaultCodeRender);
    let ctx = if let Some(man_pages_base_url) = man_pages_base_url {
        dices_print::manual::Ctx::new_with_links(renderer, man_pages_base_url)
    } else {
        dices_print::manual::Ctx::new(renderer)
    };

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
    printing: impl pretty::Pretty<'a, Arena<'a, Element>, Element>,
    out: impl std::io::Write,
) -> Result<(), Error> {
    let width = crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        // default to no wrap
        .unwrap_or(usize::MAX);

    dices_print_cli::render(printing, arena, out, width, skin.color, |element| {
        *skin.theme.style(element)
    })
    .context(PrintingSnafu)
}
