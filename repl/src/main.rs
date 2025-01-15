use std::process::ExitCode;

use dices_repl::{repl, ClapParser, ReplCli};

fn main() -> ExitCode {
    let args = ReplCli::parse();
    match repl(&args) {
        Ok(_) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
