//! Implementations of Solvable on all types of expressions

use std::num::TryFromIntError;

use derive_more::derive::{Display, Error, From};
use dices_ast::{
    expression::{
        bin_ops::{BinOp, EvalOrder},
        set::Receiver,
        Expression, ExpressionBinOp, ExpressionCall, ExpressionClosure, ExpressionList,
        ExpressionMap, ExpressionRef, ExpressionScope, ExpressionSet, ExpressionUnOp,
    },
    ident::IdentStr,
    values::{ToListError, ToNumberError, Value, ValueClosure},
};
use rand::Rng;

use crate::solve::Solvable;

#[derive(Debug, Display, Error, Clone)]
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
    #[display("`*` operator need at least one scalar")]
    MultNeedAScalar,
    #[display("Undefined variable {_0}")]
    InvalidReference(#[error(not(source))] Box<IdentStr>),
    #[display("{_0} is not callable")]
    NotCallable(#[error(not(source))] Value),
    #[display("Error during intrisic call")]
    IntrisicError(intrisics::IntrisicError),
    #[display("Closures requires {required} params, {given} were instead provided.")]
    WrongNumberOfParams { required: usize, given: usize },
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
            Expression::Set(e) => e.solve(context)?,
            Expression::Ref(e) => e.solve(context)?,
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
mod closures;
mod un_ops;
mod intrisics {
    //! Intrisic operations

    use derive_more::derive::{Display, Error};
    use dices_ast::values::{Value, ValueIntrisic};
    use rand::Rng;

    #[derive(Debug, Display, Error, Clone)]
    pub enum IntrisicError {}

    pub(super) fn call<R: Rng>(
        intrisic: ValueIntrisic,
        context: &mut crate::Context<R>,
        params: Box<[Value]>,
    ) -> Result<Value, IntrisicError> {
        todo!()
    }
}

impl Solvable for ExpressionCall {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        let Self {
            called: box called,
            params: box params,
        } = self;
        let called = called.solve(context)?;
        let params: Box<_> = params.iter().map(|p| p.solve(context)).try_collect()?;

        match called {
            Value::Intrisic(intrisic) => {
                intrisics::call(intrisic, context, params).map_err(SolveError::IntrisicError)
            }
            Value::Closure(box ValueClosure {
                params: params_names,
                captures,
                body,
            }) => {
                if params.len() != params_names.len() {
                    return Err(SolveError::WrongNumberOfParams {
                        required: params_names.len(),
                        given: params.len(),
                    });
                }
                context.jailed(|context| {
                    // adding capture vars and params
                    for (name, value) in captures.into_iter().chain(Iterator::zip(
                        params_names.into_vec().into_iter(),
                        params.into_vec(),
                    )) {
                        context.vars_mut().let_(name, value)
                    }
                    // solving in the jailed context
                    body.solve(context)
                })
            }

            _ => Err(SolveError::NotCallable(called)),
        }
    }
}

impl Solvable for ExpressionScope {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        context.scoped(|context| solve_unscoped(self, context))
    }
}

/// Solve the inner part of a scoped expression, without actually scoping
pub fn solve_unscoped<R: Rng>(
    scope: &ExpressionScope,
    context: &mut crate::Context<R>,
) -> Result<Value, SolveError> {
    for expr in &*scope.exprs {
        expr.solve(context)?;
    }
    scope.last.solve(context)
}

impl Solvable for ExpressionSet {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        let value = self.value.solve(context)?;

        match &self.receiver {
            Receiver::Ignore => (),
            Receiver::Set(box v) => {
                *context
                    .vars_mut()
                    .get_mut(v)
                    .ok_or_else(|| SolveError::InvalidReference(v.to_owned()))? = value.clone();
            }
            Receiver::Let(box v) => context.vars_mut().let_(v.to_owned(), value.clone()),
        }

        Ok(value)
    }
}
impl Solvable for ExpressionRef {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        context
            .vars()
            .get(&self.name)
            .cloned() // todo: is this clone lightweight?
            .ok_or_else(|| SolveError::InvalidReference(self.name.to_owned()))
    }
}
