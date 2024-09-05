pub(crate) use expression::solve_multiple;
use rand::Rng;

use dices_ast::{intrisics::InjectedIntr, values::Value};

use crate::Context;

mod expression;
mod value;

pub use expression::{IntrisicError, SolveError};

pub(super) trait Solvable<InjectedIntrisic: InjectedIntr> {
    type Error;

    fn solve<R: Rng>(
        &self,
        context: &mut Context<R, InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error>;
}
