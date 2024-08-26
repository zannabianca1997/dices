//! A bare-bone example repl

use std::{
    error::Error,
    io::{stdin, stdout, Write},
};

use derive_more::derive::{Display, Error, From};
use dices_ast::values::{Value, ValueNull};
use dices_engine::solve::Engine;
use rand::rngs::SmallRng;

#[derive(Debug, Clone, Error, Display, From)]
enum ReplError {
    Parse(dices_ast::parse::Error),
    Eval(dices_engine::solve::SolveError),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut context: Engine<SmallRng> = Engine::new();
    let mut line = String::new();
    loop {
        print!(">>> ");
        stdout().flush()?;
        stdin().read_line(&mut line)?;

        let res = context.eval_str(&line);

        match res {
            Ok(Value::Null(ValueNull)) => (),
            Ok(v) => println!("{v}"),
            Err(err) => {
                eprintln!("Error during evaluation:");
                eprintln!("  {err}");
                if let Some(mut src) = err.source() {
                    eprintln!();
                    eprintln!("Caused by:");
                    eprintln!("  - {src}");
                    while let Some(next_src) = src.source() {
                        src = next_src;
                        eprintln!("  - {src}");
                    }
                }
            }
        }

        line.clear();
    }
}
