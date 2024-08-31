#![feature(assert_matches)]
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(box_patterns)]

pub use context::Context;
pub use solve::SolveError;

use dices_ast::{expression::Expression, parse::parse_file, values::Value};
use either::Either::{self, Left, Right};
use nunny::NonEmpty;
use rand::{Rng, SeedableRng};
use solve::{solve_multiple, Solvable};

mod context;
mod solve;

pub struct Engine<RNG> {
    context: Context<RNG>,
}

impl<RNG> Engine<RNG> {
    /// Initialize a new engine
    ///
    /// This will use the entropy to initialize the rng
    pub fn new() -> Self
    where
        RNG: SeedableRng,
    {
        Self::new_with_rng(RNG::from_entropy())
    }

    /// Initialize a new engine
    pub fn new_with_rng(rng: RNG) -> Self {
        Self {
            context: Context::new(rng),
        }
    }

    /// Evaluate the result of an expression
    pub fn eval(&mut self, expr: &Expression) -> Result<Value, SolveError>
    where
        RNG: Rng,
    {
        expr.solve(&mut self.context)
    }

    /// Evaluate the result of multiple expressions, returning the last one
    pub fn eval_multiple(&mut self, exprs: &NonEmpty<[Expression]>) -> Result<Value, SolveError>
    where
        RNG: Rng,
    {
        solve_multiple(exprs, &mut self.context)
    }

    /// Evaluate a command string
    pub fn eval_str(
        &mut self,
        cmd: &str,
    ) -> Result<Value, Either<dices_ast::parse::Error, SolveError>>
    where
        RNG: Rng,
    {
        let exprs = parse_file(cmd).map_err(Left)?;
        self.eval_multiple(&exprs).map_err(Right)
    }
}
