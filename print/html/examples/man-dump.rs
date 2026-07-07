//! dump all manual pages as HTML

use std::{fs, path::PathBuf};

use clap::{Parser, ValueEnum};
use pretty::Arena;
use strum::Display;
use url::Url;

use dices_man::Manual;
use dices_print::{
    Pretty as _,
    manual::Ctx,
    markdown::{CodeRender, DefaultCodeRender},
};
use dices_std as _;

/// Dumps all manual pages in html format.
#[derive(Debug, Parser)]
struct Cli {
    /// Directory where to dump the manual pages
    path: PathBuf,

    /// Base url for the manual pages [default: file://{PATH}]
    #[clap(short, long)]
    base: Option<Url>,

    /// Style to render the examples
    #[clap(short, long, default_value_t)]
    example: ExampleStyle,
}

#[derive(Debug, Clone, Copy, ValueEnum, Display, Default)]
enum ExampleStyle {
    /// Evaluate the examples and render prompts + results
    #[default]
    Cli,
    /// Emit the example source verbatim
    Raw,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    fs::create_dir_all(&cli.path)?;
    // file:// url rooted at the output dir — used both as the default link base
    // and to compute each page's on-disk path via `ManPage::url`.
    let dir_base = Url::from_directory_path(cli.path.canonicalize()?)
        .expect("canonicalized dir is a valid file base");
    let link_base = cli.base.unwrap_or_else(|| dir_base.clone());

    match cli.example {
        ExampleStyle::Cli => dump_all(
            (
                dices_print_cli::examples::CliCodeRender::new(DumpPrompt),
                DefaultCodeRender,
            ),
            &link_base,
            &dir_base,
        ),
        ExampleStyle::Raw => dump_all(DefaultCodeRender, &link_base, &dir_base),
    }
}

fn dump_all<R: CodeRender>(code_render: R, link_base: &Url, dir_base: &Url) -> std::io::Result<()> {
    let manual = Manual::new();
    for page in manual.root().descendants() {
        let arena = Arena::new();
        let ctx = Ctx::new_with_links(&code_render, link_base.clone());
        let html = dices_print_html::render((&page).with_ctx(ctx), &arena)?;

        let dest = page
            .url(dir_base.clone())
            .to_file_path()
            .expect("file:// url is a valid path");
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(dest, html)?;
    }
    Ok(())
}

/// Fixed prompt strings for evaluated examples.
struct DumpPrompt;

impl dices_print_cli::PromptDisplay for DumpPrompt {
    fn prompt_left(&self) -> std::borrow::Cow<'static, str> {
        ">>".into()
    }
    fn prompt_indicator(&self) -> std::borrow::Cow<'static, str> {
        "> ".into()
    }
    fn prompt_multiline_indicator(&self) -> std::borrow::Cow<'static, str> {
        "::: ".into()
    }
}
