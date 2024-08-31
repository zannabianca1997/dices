use clap::Parser;
use dices_repl::{repl, ReplCli, ReplFatalError};

fn main() -> Result<(), ReplFatalError> {
    let args = ReplCli::parse();
    repl(args)
}
