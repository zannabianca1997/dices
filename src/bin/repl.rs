#![feature(error_reporter)]
#![feature(iter_intersperse)]

use std::{error::Report, str::FromStr};

use dices::{Cmd, ThrowsError};
use either::{Left, Right};
use rand::{thread_rng, Rng};
use rustyline::{error::ReadlineError, history::MemHistory, Config, Editor};
use thiserror::Error;

#[derive(Debug, Error)]
enum MainError {
    #[error(transparent)]
    RustyLine(#[from] ReadlineError),
    #[error("Interrupted")]
    Interrupted,
}

#[derive(Debug, Error)]
enum CmdError {
    #[error("Error while parsing command")]
    Parsing(#[source] <Cmd as FromStr>::Err),
    #[error("Error while evaluating throws")]
    Throwing(
        #[source]
        #[from]
        ThrowsError,
    ),
}

/// A command for the repl
#[derive(Debug, Clone)]
pub enum CmdOutput {
    Throw(Vec<i64>),
    Help(&'static str),
    Quit,
    None,
}

fn execute(cmd: Cmd, rng: &mut impl Rng) -> Result<CmdOutput, CmdError> {
    match cmd {
        Cmd::Throw(throw) => {
            let res = throw.throws(rng)?;
            Ok(CmdOutput::Throw(res))
        }
        Cmd::Help(topic) => Ok(CmdOutput::Help(topic.help())),
        Cmd::Quit => Ok(CmdOutput::Quit),
        Cmd::None => Ok(CmdOutput::None),
    }
}

fn main() -> Result<(), MainError> {
    let mut rl = Editor::<(), _>::with_history(Config::default(), MemHistory::new())?;
    let mut rng = thread_rng();
    println!("🎲 Welcome to DICE {} 🎲", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Input `?` to see a list of commands");
    loop {
        // Read
        let readline = rl.readline("🎲>> ");
        // Eval
        let res = match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                line.parse::<Cmd>().map_err(CmdError::Parsing)
            }
            Err(ReadlineError::Interrupted) => {
                return Err(MainError::Interrupted);
            }
            Err(ReadlineError::Eof) => {
                return Ok(());
            }
            Err(err) => {
                return Err(err.into());
            }
        }
        .and_then(|cmd| execute(cmd, &mut rng));
        // Print
        match res {
            Ok(CmdOutput::Throw(values)) => {
                print!("Results: ");
                for v in values.into_iter().map(Left).intersperse(Right(", ")) {
                    match v {
                        Left(v) => print!("{v}"),
                        Right(s) => print!("{s}"),
                    }
                }
                println!()
            }
            Ok(CmdOutput::Help(s)) => println!("{s}"),
            Ok(CmdOutput::Quit) => {
                println!("Bye!");
                return Ok(());
            }
            Ok(CmdOutput::None) => (),
            Err(err) => println!("Error: {}", Report::new(err).pretty(true)),
        }
    }
}
