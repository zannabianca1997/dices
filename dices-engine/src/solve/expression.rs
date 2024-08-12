//! Implementations of Solvable on all types of expressions

use derive_more::derive::{Display, Error, From};
use dices_ast::{
    expression::{
        bin_ops::{BinOp, EvalOrder},
        Expression, ExpressionBinOp, ExpressionCall, ExpressionClosure, ExpressionList,
        ExpressionMap, ExpressionScope, ExpressionUnOp,
    },
    values::{ToNumberError, Value},
};

use crate::Solvable;

#[derive(Debug, Display, Error)]
pub enum SolveError {
    #[display("The number of repeats must be a number")]
    RepeatTimesNotANumber(#[error(source)] ToNumberError),
    #[display("The number of repeats must be positive")]
    NegativeRepeats(#[error(source)] std::num::TryFromIntError),
}
impl From<!> for SolveError {
    fn from(value: !) -> Self {
        value
    }
}

impl Solvable for Expression {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        Ok(match self {
            Expression::Const(e) => e.solve(context)?,
            Expression::List(e) => e.solve(context)?,
            Expression::Map(e) => e.solve(context)?,
            Expression::Closure(e) => e.solve(context)?,
            Expression::UnOp(e) => e.solve(context)?,
            Expression::BinOp(e) => e.solve(context)?,
            Expression::Call(e) => e.solve(context)?,
            Expression::Scope(e) => e.solve(context)?,
        })
    }
}

impl Solvable for ExpressionList {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        Ok(Value::List(
            self.iter().map(|i| i.solve(context)).try_collect()?,
        ))
    }
}

impl Solvable for ExpressionMap {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        Ok(Value::Map(
            self.iter()
                .map(|(k, v)| v.solve(context).map(|v| (k.clone(), v)))
                .try_collect()?,
        ))
    }
}

impl Solvable for ExpressionBinOp {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        let ExpressionBinOp {
            op,
            expressions: box [a, b],
        } = self;
        let [a, b] = match op.eval_order() {
            Some(EvalOrder::AB) => {
                let a = a.solve(context)?;
                let b = b.solve(context)?;
                [a, b]
            }
            Some(EvalOrder::BA) => {
                let b = b.solve(context)?;
                let a = a.solve(context)?;
                [a, b]
            }
            None => {
                let BinOp::Repeat = op else {
                    unreachable!("The only special order should be `Repeat`")
                };

                // finding out the number of repeats
                let repeats: i64 = b
                    .solve(context)?
                    .to_number()
                    .map_err(SolveError::RepeatTimesNotANumber)?
                    .into();
                let repeats: u64 = repeats
                    .try_into()
                    .map_err(|err| SolveError::NegativeRepeats(err))?;

                return Ok(Value::List(
                    (0..repeats).map(|_| a.solve(context)).try_collect()?,
                ));
            }
        };
        todo!()
    }
}
impl Solvable for ExpressionUnOp {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        todo!()
    }
}

impl Solvable for ExpressionClosure {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        todo!()
    }
}

impl Solvable for ExpressionCall {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        todo!()
    }
}

impl Solvable for ExpressionScope {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        context.scoped(|context| {
            for expr in &*self.exprs {
                expr.solve(context)?;
            }
            self.last.solve(context)
        })
    }
}
