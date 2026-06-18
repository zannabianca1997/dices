#![doc = include_str!("../README.md")]

use std::io::{self, stdout};

use crossterm::terminal;
use dices_engine::Engine;
use pretty::{
    Arena, Pretty, TermColored,
    termcolor::{self, Ansi, NoColor},
};
use rand_seeder::Seeder;
use reedline::{Reedline, Signal};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::{
    cli::Cli,
    config::{Config, skin::SelectedSkin},
};

pub mod cli;
pub mod config;
mod history;
mod prompt;
mod banners {
    use pretty::{
        DocAllocator, Pretty,
        termcolor::{Color, ColorSpec},
    };

    use crate::config::skin::Skin;

    pub struct OpeningBanner<'a>(pub &'a Skin);

    impl<'a, D> Pretty<'a, D, ColorSpec> for OpeningBanner<'_>
    where
        D: DocAllocator<'a, ColorSpec>,
    {
        fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, ColorSpec> {
            let code = ColorSpec::new()
                .set_bg(Some(Color::Black))
                .set_dimmed(true)
                .clone();

            allocator
                .nil()
                .append(self.0.emoji.then_some("⛓️🐉 ~ "))
                .append("Welcome to ")
                .append(
                    allocator
                        .text("dices ")
                        .append(env!("CARGO_PKG_VERSION"))
                        .annotate(code.clone()),
                )
                .append(self.0.emoji.then_some(" ~ ⛓️🐉"))
                .annotate(
                    ColorSpec::new()
                        .set_bold(true)
                        .set_fg(Some(Color::Magenta))
                        .clone(),
                )
                .append(allocator.hardline())
                .append(allocator.hardline())
                .append(allocator.concat([
                    allocator.text("Use "),
                    allocator.text("help()").annotate(code.clone()),
                    allocator.text(" for the manual, and "),
                    allocator.text("quit()").annotate(code.clone()),
                    allocator.text(" or "),
                    allocator.text("Ctrl+D").annotate(code.clone()),
                    allocator.text(" to exit."),
                ]))
                .append(allocator.hardline())
        }
    }

    pub struct ClosingBanner<'a>(pub &'a Skin);

    impl<'a, D> Pretty<'a, D, ColorSpec> for ClosingBanner<'_>
    where
        D: DocAllocator<'a, ColorSpec>,
    {
        fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, ColorSpec> {
            allocator
                .nil()
                .append(self.0.emoji.then_some("⛓️🐉 ~ "))
                .append("See you at the next game!")
                .append(self.0.emoji.then_some(" ~ ⛓️🐉"))
                .annotate(
                    ColorSpec::new()
                        .set_bold(true)
                        .set_fg(Some(Color::Magenta))
                        .clone(),
                )
                .append(allocator.hardline())
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    Config { source: figment::Error },
    #[snafu(display("Error while printing"))]
    Printing { source: io::Error },
    #[snafu(display("Error while reading command"))]
    ReadLine { source: io::Error },
    #[snafu(display("Error while handling history"))]
    History { source: reedline::ReedlineError },
    #[snafu(display("Unknown skin {skin}"))]
    UnknownSkin { skin: String },
    #[snafu(display("Aborted."))]
    Aborted,
}

pub fn main(Cli { config, seed }: Cli) -> Result<(), Error> {
    let Config {
        history,
        skin,
        skins,
    } = Config::extract(config)?;
    let skin = skins.get(&skin).with_context(|| {
        let SelectedSkin::Other(skin) = skin else {
            unreachable!()
        };
        UnknownSkinSnafu { skin }
    })?;

    // Init the engine
    let mut engine = Engine::new(Seeder::from(seed).make_seed());

    // Prepare the repl
    let mut line_editor = Reedline::create()
        .with_history(Box::new(history::history(history).context(HistorySnafu)?))
        .with_ansi_colors(skin.ansi);
    let prompt = prompt::Prompt(skin);

    if skin.banners {
        let arena = Arena::new();
        let banner = banners::OpeningBanner(skin).pretty(&arena);
        let width = crossterm::terminal::size().map(|(c, _)| c).unwrap_or(80) as _;
        match skin.ansi {
            true => banner
                .render_colored(width, &mut Ansi::new(stdout()))
                .context(PrintingSnafu)?,
            false => banner
                .render_colored(width, &mut NoColor::new(stdout()))
                .context(PrintingSnafu)?,
        }
    }

    // Main repl cycle
    loop {
        let sig = line_editor.read_line(&prompt).context(ReadLineSnafu)?;
        match sig {
            Signal::Success(buffer) => {
                println!("Read: {}", buffer);
            }
            Signal::CtrlD => break,
            Signal::CtrlC => return Err(Error::Aborted),
            Signal::HostCommand(_) => unreachable!(),
            _ => panic!("Unhandled signal"),
        }
    }

    if skin.banners {
        let arena = Arena::new();
        let banner = banners::ClosingBanner(skin).pretty(&arena);
        let width = crossterm::terminal::size().map(|(c, _)| c).unwrap_or(80) as _;
        match skin.ansi {
            true => banner
                .render_colored(width, Ansi::new(stdout()))
                .context(PrintingSnafu)?,
            false => banner
                .render_colored(width, NoColor::new(stdout()))
                .context(PrintingSnafu)?,
        }
    }

    Ok(())
}
