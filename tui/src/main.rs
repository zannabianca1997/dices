//! Entry point of the TUI

use std::process::ExitCode;

use clap::Parser;

use dices_tui::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    dices_tui::main_print_error(cli)
}
