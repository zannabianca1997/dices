use bin_ops::{add, mult};
use dices_ast::expression::un_ops::UnOp;
use itertools::Itertools;
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

pub(crate) fn plus<R>(context: &mut crate::Context<R>, a: Value) -> Result<Value, SolveError> {
    Ok(match a {
        // scalars will be converted to numbers
        Value::Null(_)
        | Value::Bool(_)
        | Value::Number(_)
        | Value::String(_)
        | Value::Intrisic(_)
        | Value::Closure(_) => a
            .to_number()
            .map_err(|source| SolveError::CannotMakeANumber { source })?
            .into(),
        // List and maps are summed recursively
        Value::List(l) => l
            .into_iter()
            .map(Ok)
            .tree_reduce(|a, b| add(context, a?, b?))
            .transpose()?
            .unwrap_or(Value::Number(0.into())),
        Value::Map(m) => m
            .into_iter()
            .map(|(_, v)| Ok(v))
            .tree_reduce(|a, b| add(context, a?, b?))
            .transpose()?
            .unwrap_or(Value::Number(0.into())),
    })
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
