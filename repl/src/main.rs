#![feature(error_reporter)]

use std::error::Report;

use dices_repl::{repl, ClapParser, ReplCli, ReplFatalError};

fn main() -> Result<(), Report<ReplFatalError>> {
    let args = ReplCli::parse();
    repl(args).map_err(|err| Report::new(err).pretty(true))
}
