//! Entry point of the CLI

use std::process::ExitCode;

use clap::Parser;

use dices_cli::cli::Cli;

fn main() -> ExitCode {
    let cli = Cli::parse();
    dices_cli::main_print_error(cli)
}
