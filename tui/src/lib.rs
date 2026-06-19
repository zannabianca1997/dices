#![doc = include_str!("../README.md")]

use std::{
    hash::Hash,
    io::{self},
    process::{ExitCode, Termination},
};

use dices_engine::{Engine, Evaluator};
use dices_values::{Value, null::ValueNull};
use rand_seeder::Seeder;
use reedline::{Reedline, Signal};
use snafu::{ResultExt, Snafu};

use crate::{
    cli::Cli,
    config::Config,
    print::{print_error, print_markdown, print_value},
};

mod banners;
pub mod cli;
mod config;
mod history;
pub mod print;
mod prompt;

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

#[derive(Debug, Snafu)]
pub enum CommandError {
    #[snafu(transparent)]
    Parse { source: dices_parser::ParseError },
    #[snafu(transparent)]
    Eval { source: dices_engine::EvalError },
}

fn main_inner(seed: impl Hash, Config { history, skin }: &Config) -> Result<(), Error> {
    // Init the engine
    let mut engine = Engine::new(Seeder::from(seed).make_seed());

    // Prepare the repl
    let mut line_editor = Reedline::create()
        .with_history(Box::new(history::history(history).context(HistorySnafu)?))
        .with_ansi_colors(skin.color);
    let prompt = prompt::Prompt(&skin);

    if skin.banners {
        print_markdown(&skin, banners::OPENING)?;
    }

    // Main repl cycle
    loop {
        // Read
        let read = match line_editor.read_line(&prompt).context(ReadLineSnafu)? {
            Signal::Success(buffer) => buffer,
            Signal::CtrlD => break,
            Signal::CtrlC => return Err(Error::Aborted),
            Signal::HostCommand(_) => unreachable!(),
            _ => panic!("Unhandled signal"),
        };

        // Eval
        let eval = dices_parser::parse_statement(&read.into())
            .map_err(CommandError::from)
            .and_then(|stmt| engine.eval(&stmt).map_err(CommandError::from));

        // Print
        match eval {
            Ok(Value::Null(ValueNull)) => (),
            Ok(value) => print_value(&skin, value)?,
            Err(error) => print_error(&skin, &error)?,
        }
    }

    if skin.banners {
        print_markdown(&skin, banners::CLOSING)?;
    }

    Ok(())
}

pub fn main(Cli { config, seed }: Cli) -> Result<(), Error> {
    let config = Config::extract(config)?;
    main_inner(seed, &config)
}
pub fn main_print_error(Cli { config, seed }: Cli) -> ExitCode {
    let config = match Config::extract(config) {
        Ok(c) => c,
        Err(err) => {
            // No skin, need to directly print
            // using snafu reporter
            return snafu::Report::from_error(err).report();
        }
    };

    match main_inner(seed, &config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            print_error(&config.skin, &err).unwrap_or_else(|_| {
                // Failed to print error nicely, print directly
                snafu::Report::from_error(err).report();
            });
            ExitCode::FAILURE
        }
    }
}
