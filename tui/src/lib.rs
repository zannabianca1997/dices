#![doc = include_str!("../README.md")]

use std::{
    hash::Hash,
    io::{self},
    process::{ExitCode, Termination},
};

use dices_engine::{Engine, Evaluator};
use dices_man::ManItem;
use dices_std::Std;
use dices_values::{Value, null::ValueNull, string::ValueString};
use itertools::Itertools;
use rand::{Rng, rngs::OsRng};
use rand_seeder::Seeder;
use reedline::{Reedline, Signal};
use snafu::{ResultExt, Snafu};

use crate::{
    cli::Cli,
    config::{Config, skin::Skin},
    print::{print_error, print_man_item, print_markdown, print_value},
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

struct Ui<'a>(&'a Skin);

impl dices_engine::ui::Ui for Ui<'_> {
    fn print(&self, value: impl Into<Value>) -> Result<(), Self::PrintError> {
        print_value(self.0, value.into()).unwrap();
        println!();
        Ok(())
    }

    fn manual(&self, item: &ManItem) -> Result<(), Self::PrintError> {
        print_man_item(self.0, &item)
    }

    type PrintError = Error;

    fn print_str<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError> {
        print!("{}", value.as_ref());
        if !value.as_ref().ends_with('\n') {
            println!()
        }
        Ok(())
    }

    fn print_md<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError> {
        print_markdown(self.0, value.as_ref())
    }
}

fn main_inner(
    seed: Option<impl Hash>,
    Config {
        history,
        skin,
        std: std_opts,
    }: &Config,
    interactive: bool,
    command: Option<Vec<String>>,
) -> Result<(), Error> {
    // Init the engine
    let mut engine = Engine::new(
        if let Some(seed) = seed {
            Seeder::from(seed).make_seed()
        } else {
            OsRng.r#gen()
        },
        Std::new(std_opts.clone()),
    );

    if skin.banners {
        print_markdown(skin, banners::OPENING)?;
    }

    // Execute command
    if let Some(command) = command.as_ref() {
        // Merge args into a single command
        let command = command.iter().join(" ");

        // Eval
        let eval = dices_parser::parse_scope_inner(&command.into())
            .map_err(CommandError::from)
            .and_then(|scope_inner| {
                engine
                    .eval(&scope_inner, Ui(skin))
                    .map_err(CommandError::from)
            });

        // Print
        match eval {
            Ok(Value::Null(ValueNull)) => (),
            Ok(value) => print_value(skin, value)?,
            Err(error) => print_error(skin, &error)?,
        }
        println!()
    }

    if command.is_none_or(|_| interactive) {
        // Prepare the repl
        let mut line_editor = Reedline::create()
            .with_history(Box::new(history::history(history).context(HistorySnafu)?))
            .with_ansi_colors(skin.color);
        let prompt = prompt::Prompt(skin);

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
            let eval = dices_parser::parse_scope_inner(&read.into())
                .map_err(CommandError::from)
                .and_then(|scope_inner| {
                    engine
                        .eval(&scope_inner, Ui(skin))
                        .map_err(CommandError::from)
                });

            // Print
            match eval {
                Ok(Value::Null(ValueNull)) => (),
                Ok(value) => print_value(skin, value)?,
                Err(error) => print_error(skin, &error)?,
            }
        }
    }

    if skin.banners {
        print_markdown(skin, banners::CLOSING)?;
    }

    Ok(())
}

pub fn main(
    Cli {
        config,
        seed,
        interactive,
        command,
    }: Cli,
) -> Result<(), Error> {
    let config = Config::extract(config, command.is_some())?;
    main_inner(seed, &config, interactive, command)
}
pub fn main_print_error(
    Cli {
        config,
        seed,
        interactive,
        command,
    }: Cli,
) -> ExitCode {
    let config = match Config::extract(config, command.is_some()) {
        Ok(c) => c,
        Err(err) => {
            // No skin, need to directly print
            // using snafu reporter
            return snafu::Report::from_error(err).report();
        }
    };

    match main_inner(seed, &config, interactive, command) {
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
