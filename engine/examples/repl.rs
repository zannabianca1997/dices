//! A very bare bone repl
#![feature(error_reporter)]

use std::{error::Report, fmt::Debug};

use engine::{expr::EvalError, namespace::Namespace, parser::parse_statement, value::Value};
use peg::{error::ParseError, str::LineCol};
use pretty::{Arena, DocAllocator, Pretty};
use rand::{rngs::SmallRng, SeedableRng};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Parse(ParseError<LineCol>),
    #[error(transparent)]
    Eval(EvalError),
}

fn main() -> rustyline::Result<()> {
    let mut rl = rustyline::DefaultEditor::new()?;
    let mut namespace = Namespace::root();
    let mut rng = SmallRng::from_entropy();
    'repl: loop {
        // Read
        let line = match rl.readline(">> ") {
            Ok(line) => line,
            // iterrupted is not a error!
            Err(rustyline::error::ReadlineError::Interrupted) => break 'repl,
            Err(err) => return Err(err),
        };
        // Eval
        let res = parse_statement(&line)
            .map_err(Error::Parse)
            .and_then(|stm| stm.eval(&mut namespace, &mut rng).map_err(Error::Eval));
        // Print
        match res {
            Ok(Value::Null) => (),
            Ok(val) => {
                // we sadly have to allocate a new arena for every value we print, as there is no way of guarantee that
                // the arena is empty after the printing
                let docs_arena = Arena::<()>::new();
                // now we render the result
                let doc = &*val.pretty(&docs_arena).append(docs_arena.hardline());
                print!("{}", doc.pretty(80))
            }
            Err(err) => eprintln!("{}", Report::new(err).pretty(true)),
        }
        // Loop
        rl.add_history_entry(line)?;
    }
    println!("Bye!");
    Ok(())
}
