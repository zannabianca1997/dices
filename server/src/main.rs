#![feature(error_reporter)]

use std::error::Report;

use dices_server::{ClapParser, Cli, MainError};

fn main() -> Result<(), Report<MainError>> {
    dices_server::main(Cli::parse()).map_err(|err| Report::new(err).pretty(true))
}
