#![doc = include_str!("../README.md")]

use std::io;

use dices_engine::Engine;
use rand_seeder::Seeder;
use reedline::{Reedline, Signal};
use snafu::{OptionExt, ResultExt, Snafu};

use crate::{
    cli::Cli,
    config::{Config, skin::SelectedSkin},
    history::history,
};

pub mod cli;
pub mod config;
mod prompt;
mod history {

    use reedline::{FileBackedHistory, History};

    use crate::config::history::HistoryConfig;

    pub fn history(config: HistoryConfig) -> reedline::Result<impl History> {
        if let Some(file) = config.file() {
            FileBackedHistory::with_file(config.capacity, file)
        } else {
            FileBackedHistory::new(config.capacity)
        }
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(transparent)]
    Config { source: figment::Error },
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

    Ok(())
}
