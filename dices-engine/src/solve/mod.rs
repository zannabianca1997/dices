use rand::Rng;

use dices_ast::values::Value;

use crate::Context;

mod expression;
mod value;

pub use expression::{solve_unscoped, SolveError};

pub trait Solvable {
    type Error;

    fn solve<R: Rng>(&self, context: &mut Context<R>) -> Result<Value, Self::Error>;
}
