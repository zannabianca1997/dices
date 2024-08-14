use bin_ops::{add, mult};
use dices_ast::expression::un_ops::UnOp;
use itertools::concat;
use rand::Rng;

use super::*;

impl Solvable for ExpressionUnOp {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        let ExpressionUnOp {
            op,
            expression: box a,
        } = self;
        let a = a.solve(context)?;
        Ok(match op {
            UnOp::Plus => plus,
            UnOp::Neg => neg,
            UnOp::Dice => dice,
        }(context, a)?)
    }
}

fn plus<R>(context: &mut crate::Context<R>, a: Value) -> Result<Value, SolveError> {
    // delegating to the binary plus
    add(context, Value::Number(0.into()), a)
}

pub(super) fn neg<R>(context: &mut crate::Context<R>, a: Value) -> Result<Value, SolveError> {
    // delegating to the mult op
    mult(context, a, Value::Number((-1).into()))
}

fn dice<R: Rng>(context: &mut crate::Context<R>, a: Value) -> Result<Value, SolveError> {
    let a: i64 = a
        .to_number()
        .map_err(|source| SolveError::FacesAreNotANumber { source })?
        .into();

    let f: usize = a
        .try_into()
        .map_err(|source| SolveError::FacesMustBePositive { source })?;
    Ok(Value::Number((context.rng().gen_range(1..f) as i64).into()))
}
