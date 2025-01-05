pub(crate) use expression::solve_multiple;

use dices_ast::{intrisics::InjectedIntr, value::Value};

use crate::{Context, DicesRng};

mod expression;
mod value;

pub use expression::{IntrisicError, SolveError};

pub(super) trait Solvable<InjectedIntrisic: InjectedIntr> {
    type Error;

    fn solve<R: DicesRng>(
        &self,
        context: &mut Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error>;
}
