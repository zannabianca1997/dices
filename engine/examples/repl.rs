//! A very bare bone repl
#![feature(error_reporter)]

use std::{
    error::Report,
    io::{self, stdin},
};

use engine::{expr::EvalError, namespace::Namespace, parser::parse_statement};
use peg::{error::ParseError, str::LineCol};
use rand::{rngs::SmallRng, SeedableRng};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    IO(io::Error),
    #[error(transparent)]
    Parse(ParseError<LineCol>),
    #[error(transparent)]
    Eval(EvalError),
}

fn main() {
    let mut buf = String::new();
    let mut namespace = Namespace::root();
    let mut rng = SmallRng::from_entropy();
    loop {
        // Read
        buf.clear();
        let line = stdin()
            .read_line(&mut buf)
            .map(|_| &*buf)
            .map_err(Error::IO);
        // Eval
        let stm = line.and_then(|stm| parse_statement(stm).map_err(Error::Parse));
        let res = stm.and_then(|stm| stm.eval(&mut namespace, &mut rng).map_err(Error::Eval));
        // Print
        match res {
            Ok(val) => println!("{val:#?}"),
            Err(err) => eprintln!("{}", Report::new(err).pretty(true)),
        }
        // Loop
    }
}
