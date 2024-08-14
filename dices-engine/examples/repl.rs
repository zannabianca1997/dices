//! A bare-bone example repl

use std::{
    error::Error,
    io::{stdin, stdout, Write},
};

use derive_more::derive::{Display, Error, From};
use dices_ast::{
    parse::parse_file,
    values::{Value, ValueNull},
};
use dices_engine::{
    solve::{solve_unscoped, Solvable},
    Context,
};
use rand::{rngs::SmallRng, SeedableRng};

#[derive(Debug, Clone, Error, Display, From)]
enum ReplError {
    Parse(dices_ast::parse::Error),
    Eval(dices_engine::solve::SolveError),
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut context = Context::new(SmallRng::from_entropy());
    let mut line = String::new();
    loop {
        print!(">>> ");
        stdout().flush()?;
        stdin().read_line(&mut line)?;

        let expr = parse_file(&line).map_err(ReplError::from);
        let res =
            expr.and_then(|expr| solve_unscoped(&expr, &mut context).map_err(ReplError::from));

        match res {
            Ok(Value::Null(ValueNull)) => (),
            Ok(v) => println!("{v}"),
            Err(err) => eprintln!("{err}"),
        }

        line.clear();
    }
}
