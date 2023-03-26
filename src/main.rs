#![feature(error_reporter)]
#![feature(iter_intersperse)]

use std::error::Report;

use rand::thread_rng;
use rustyline::{error::ReadlineError, history::MemHistory, Config, Editor};
use termimad::{minimad::TextTemplate, Alignment, FmtText, MadSkin};
use thiserror::Error;

use dices::{Cmd, CmdError};

#[derive(Debug, Error)]
enum MainError {
    #[error(transparent)]
    RustyLine(#[from] ReadlineError),
    #[error("Interrupted")]
    Interrupted,
}

fn main() -> Result<(), MainError> {
    let mut rl = Editor::<(), _>::with_history(Config::default(), MemHistory::new())?;
    let skin = {
        let mut s = MadSkin::default();
        s.headers[0].align = Alignment::Left;
        s
    };
    let mut rng = thread_rng();

    let header_template = TextTemplate::from(include_str!("header.md"));
    let mut header_expander = header_template.expander();
    header_expander
        .set("version", env!("CARGO_PKG_VERSION"))
        .set("name", env!("CARGO_PKG_NAME"));
    let header = FmtText::from_text(&skin, header_expander.expand(), None);
    print!("{header}");

    loop {
        // Read
        let readline = rl.readline("🎲 >> ");
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
        .and_then(|cmd| cmd.execute(&mut rng));
        // Print
        match res {
            Ok(output) => {
                output.print(&skin);
                if output.is_final() {
                    return Ok(());
                }
            }
            Err(err) => println!("Error: {}", Report::new(err).pretty(true)),
        }
    }
}
