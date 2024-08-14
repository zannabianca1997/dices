//! Implementations of Solvable on all types of expressions

use std::num::TryFromIntError;

use derive_more::derive::{Display, Error};
use dices_ast::{
    expression::{
        bin_ops::{BinOp, EvalOrder},
        Expression, ExpressionBinOp, ExpressionCall, ExpressionClosure, ExpressionList,
        ExpressionMap, ExpressionScope, ExpressionUnOp,
    },
    values::{ToListError, ToNumberError, Value},
};
use rand::Rng;

use crate::Solvable;

#[derive(Debug, Display, Error)]
pub enum SolveError {
    #[display("The number of repeats must be a number")]
    RepeatTimesNotANumber(#[error(source)] ToNumberError),
    #[display("The number of repeats must be positive")]
    NegativeRepeats(#[error(source)] TryFromIntError),
    #[display("The operator {} needs a number at is right", op)]
    RHSIsNotANumber {
        op: BinOp,
        #[error(source)]
        source: ToNumberError,
    },
    #[display("The operator {} needs a number at is left", op)]
    LHSIsNotANumber {
        op: BinOp,
        #[error(source)]
        source: ToNumberError,
    },
    #[display("The operator {} needs a list at is right", op)]
    RHSIsNotAList {
        op: BinOp,
        #[error(source)]
        source: ToListError,
    },
    #[display("The operator {} needs a list at is left", op)]
    LHSIsNotAList {
        op: BinOp,
        #[error(source)]
        source: ToListError,
    },
    #[display("Integer overflow")]
    Overflow,
    #[display("The filter operator {} needs a list of number at his left", op)]
    FilterNeedNumber {
        op: BinOp,
        #[error(source)]
        source: ToNumberError,
    },
    #[display("The filter operator {} needs a positive number at his right", op)]
    FilterNeedPositive {
        op: BinOp,
        #[error(source)]
        source: TryFromIntError,
    },
    #[display("The number of dice faces must be a number")]
    FacesAreNotANumber {
        #[error(source)]
        source: ToNumberError,
    },
    #[display("The number of dice faces must be positive")]
    FacesMustBePositive {
        #[error(source)]
        source: TryFromIntError,
    },
    #[display("Cannot convert into a number")]
    CannotMakeANumber {
        #[error(source)]
        source: ToNumberError,
    },
}
impl From<!> for SolveError {
    fn from(value: !) -> Self {
        value
    }
}

impl Solvable for Expression {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
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

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        Ok(Value::List(
            self.iter().map(|i| i.solve(context)).try_collect()?,
        ))
    }
}

impl Solvable for ExpressionMap {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        Ok(Value::Map(
            self.iter()
                .map(|(k, v)| v.solve(context).map(|v| (k.clone(), v)))
                .try_collect()?,
        ))
    }
}

mod bin_ops;
mod un_ops;

impl Solvable for ExpressionClosure {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        todo!()
    }
}

impl Solvable for ExpressionCall {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        todo!()
    }
}

impl Solvable for ExpressionScope {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        context.scoped(|context| {
            for expr in &*self.exprs {
                expr.solve(context)?;
            }
            self.last.solve(context)
        })
    }
}
