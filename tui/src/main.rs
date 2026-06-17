//! Entry point of the TUI

use clap::Parser;

use dices_tui::{Error, cli::Cli};

#[snafu::report]
fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    dices_tui::main(cli)
}
