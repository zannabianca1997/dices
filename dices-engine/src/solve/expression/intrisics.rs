//! Intrisic operations

use std::str::FromStr;

use derive_more::{Display, Error};
use dices_ast::{
    expression::{bin_ops::BinOp, Expression, ExpressionBinOp, ExpressionCall},
    intrisics::{InjectedIntr, Intrisic},
    values::{ToListError, ToNumberError, Value, ValueIntrisic},
};
use rand::Rng;

use crate::solve::Solvable;

use super::SolveError;

#[derive(Debug, Display, Error, Clone)]
pub enum IntrisicError<Injected>
where
    Injected: InjectedIntr,
{
    #[display("Wrong number of params given to the intrisic {}: expected {}, given {given}", called.name(), param_num(called))]
    WrongParamNum {
        called: Intrisic<Injected>,
        given: usize,
    },
    #[display("Expression called failed to evaluate")]
    CallFailed(#[error(source)] SolveError<Injected>),
    #[display("Error during summing")]
    SumFailed(#[error(source)] SolveError<Injected>),
    #[display("Error during multiplying")]
    MultFailed(#[error(source)] SolveError<Injected>),
    #[display("Error during joining")]
    JoinFailed(#[error(source)] SolveError<Injected>),
    #[display("The second parameter of `call` must be a list of parameters")]
    CallParamsNotAList(#[error(source)] ToListError),
    #[display("Cannot convert to a number")]
    ToNumber(#[error(source)] ToNumberError),
    #[display("Cannot convert to a list")]
    ToList(#[error(source)] ToListError),
    #[display("`parse` must be called on a string, not on {_0}")]
    CannotParseNonString(#[error(not(source))] Value<Injected>),
    #[display("Failed to parse string")]
    ParseFailed(#[error(source)] <Value<Injected> as FromStr>::Err),

    #[display("{_0}")]
    Injected(#[error(source)] Injected::Error),
}

pub(super) fn call<R: Rng, Injected>(
    intrisic: ValueIntrisic<Injected>,
    context: &mut crate::Context<R, Injected>,
    params: Box<[Value<Injected>]>,
) -> Result<Value<Injected>, IntrisicError<Injected>>
where
    Injected: InjectedIntr,
{
    match intrisic.into() {
        // Variadics
        Intrisic::Call => {
            let [called, params] = match Box::<[_; 2]>::try_from(params) {
                Ok(box [a, b]) => [a, b],
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::Call,
                        given: s.len(),
                    })
                }
            };

            ExpressionCall {
                called: Box::new(called.into()),
                params: params
                    .to_list()
                    .map_err(IntrisicError::CallParamsNotAList)?
                    .into_iter()
                    .map(Expression::from)
                    .collect(),
            }
            .solve(context)
            .map_err(IntrisicError::CallFailed)
        }
        Intrisic::Sum => params
            .into_vec()
            .into_iter()
            .try_reduce(|e1, e2| {
                Expression::BinOp(ExpressionBinOp::new(BinOp::Add, e1.into(), e2.into()))
                    .solve(context)
            })
            .map(|r| r.unwrap_or(Value::Number(0.into())))
            .map_err(IntrisicError::SumFailed),
        Intrisic::Join => params
            .into_vec()
            .into_iter()
            .try_reduce(|e1, e2| {
                Expression::BinOp(ExpressionBinOp::new(BinOp::Join, e1.into(), e2.into()))
                    .solve(context)
            })
            .map(|r| r.unwrap_or(Value::List([].into_iter().collect())))
            .map_err(IntrisicError::JoinFailed),
        Intrisic::Mult => params
            .into_vec()
            .into_iter()
            .try_reduce(|e1, e2| {
                Expression::BinOp(ExpressionBinOp::new(BinOp::Mult, e1.into(), e2.into()))
                    .solve(context)
            })
            .map(|r| r.unwrap_or(Value::Number(0.into())))
            .map_err(IntrisicError::MultFailed),

        // Conversions
        Intrisic::ToNumber => {
            let [value] = match Box::<[_; 1]>::try_from(params) {
                Ok(box [a]) => [a],
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::ToNumber,
                        given: s.len(),
                    })
                }
            };
            value
                .to_number()
                .map(Into::into)
                .map_err(IntrisicError::ToNumber)
        }
        Intrisic::ToList => {
            let [value] = match Box::<[_; 1]>::try_from(params) {
                Ok(box [a]) => [a],
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::ToList,
                        given: s.len(),
                    })
                }
            };
            value
                .to_list()
                .map(Into::into)
                .map_err(IntrisicError::ToList)
        }
        Intrisic::ToString => {
            let [value] = match Box::<[_; 1]>::try_from(params) {
                Ok(box [a]) => [a],
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::ToString,
                        given: s.len(),
                    })
                }
            };
            Ok(Value::String(value.to_string().into()))
        }
        Intrisic::Parse => {
            let [value] = match Box::<[_; 1]>::try_from(params) {
                Ok(box [Value::String(s)]) => [s],
                Ok(box [a]) => return Err(IntrisicError::CannotParseNonString(a)),
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::Parse,
                        given: s.len(),
                    })
                }
            };
            value.trim().parse().map_err(IntrisicError::ParseFailed)
        }
        Intrisic::Injected(injected) => injected
            .call(context.injected_intrisics_data_mut(), params)
            .map_err(IntrisicError::Injected),
    }
}

fn param_num<Injected>(intr: &Intrisic<Injected>) -> usize {
    match intr {
        Intrisic::Call => 2,
        Intrisic::ToString | Intrisic::Parse | Intrisic::ToNumber | Intrisic::ToList => 1,
        Intrisic::Sum | Intrisic::Join | Intrisic::Mult | Intrisic::Injected(_) => {
            panic!("These have no fixed param number")
        }
    }
}
