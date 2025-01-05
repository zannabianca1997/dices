use bin_ops::{add, mult};
use dices_ast::expression::un_ops::UnOp;
use itertools::Itertools;
use rand::Rng;

use super::*;

impl<InjectedIntrisic> Solvable<InjectedIntrisic> for ExpressionUnOp<InjectedIntrisic>
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, SolveError<InjectedIntrisic>> {
        let ExpressionUnOp {
            op,
            expression: box a,
        } = self;
        let a = a.solve(context)?;
        (match op {
            UnOp::Plus => plus,
            UnOp::Neg => neg,
            UnOp::Dice => dice,
        }(context, a))
    }
}

pub(crate) fn plus<R, InjectedIntrisic: InjectedIntr>(
    context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    a: Value<InjectedIntrisic>,
) -> Result<Value<InjectedIntrisic>, SolveError<InjectedIntrisic>> {
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

pub(super) fn neg<R, InjectedIntrisic>(
    context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    a: Value<InjectedIntrisic>,
) -> Result<Value<InjectedIntrisic>, SolveError<InjectedIntrisic>>
where
    InjectedIntrisic: InjectedIntr,
{
    // delegating to the mult op
    mult(context, Value::Number((-1).into()), a)
}

fn dice<R: Rng, InjectedIntrisic: InjectedIntr>(
    context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    a: Value<InjectedIntrisic>,
) -> Result<Value<InjectedIntrisic>, SolveError<InjectedIntrisic>> {
    let a = a
        .to_number()
        .map_err(|source| SolveError::FacesAreNotANumber { source })?;

    if a <= ValueNumber::ZERO {
        return Err(SolveError::FacesMustBePositive { faces: a });
    }

    Ok(Value::Number(
        context.rng().gen_range(ValueNumber::from(1)..=a),
    ))
}
