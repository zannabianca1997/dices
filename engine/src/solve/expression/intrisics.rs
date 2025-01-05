//! Intrisic operations

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    str::FromStr,
};

use derive_more::{Display, Error};
use dices_ast::{
    expression::{bin_ops::BinOp, Expression, ExpressionBinOp, ExpressionCall},
    intrisics::{InjectedIntr, Intrisic},
    value::{
        serde::{deserialize_from_value, serialize_to_value},
        ToListError, ToNumberError, Value, ValueIntrisic, ValueNull,
    },
};
use rand::SeedableRng;

use crate::{solve::Solvable, DicesRng};

use super::SolveError;

#[derive(Debug, Display, Error)]
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
    #[display("`from_json` must be called on a string, not on {_0}")]
    JsonMustBeString(#[error(not(source))] Value<Injected>),
    #[display("Failed to parse string")]
    ParseFailed(#[error(source)] <Value<Injected> as FromStr>::Err),

    #[display("{_0}")]
    Injected(#[error(source)] Injected::Error),
    #[display("Cannot deserialize from json")]
    JsonError(#[error(source)] serde_json::Error),
    #[display("Invalid RNG state")]
    InvalidRngState(#[error(source)] dices_ast::value::serde::DeserializeFromValueError),
}

pub(super) fn call<R: DicesRng, Injected>(
    intrisic: ValueIntrisic<Injected>,
    context: &mut crate::Context<R, Injected, Injected::Data>,
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
            .map(|r| r.unwrap_or(Value::Number(1.into())))
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

        Intrisic::ToJson => {
            let [value] = match Box::<[_; 1]>::try_from(params) {
                Ok(box [v]) => [v],
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::ToJson,
                        given: s.len(),
                    })
                }
            };
            serde_json::to_string(&value)
                .map(|s| Value::String(s.into()))
                .map_err(IntrisicError::JsonError)
        }
        Intrisic::FromJson => {
            let [value] = match Box::<[_; 1]>::try_from(params) {
                Ok(box [Value::String(s)]) => [s],
                Ok(box [a]) => return Err(IntrisicError::JsonMustBeString(a)),
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::FromJson,
                        given: s.len(),
                    })
                }
            };
            serde_json::from_str(&value).map_err(IntrisicError::JsonError)
        }

        Intrisic::SeedRNG => {
            *context.rng() = if params.is_empty() {
                // if no parameter is given, seed from entropy
                SeedableRng::from_entropy()
            } else {
                // Hash all the parameters
                let mut hasher = DefaultHasher::new();
                params.hash(&mut hasher);
                SeedableRng::seed_from_u64(hasher.finish())
            };
            Ok(Value::Null(ValueNull))
        }
        Intrisic::SaveRNG => Ok(serialize_to_value(context.rng())
            .expect("The RNG should be always serializable to a value")),
        Intrisic::RestoreRNG => {
            let [value] = match Box::<[_; 1]>::try_from(params) {
                Ok(box [v]) => [v],
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::RestoreRNG,
                        given: s.len(),
                    })
                }
            };
            *context.rng() =
                deserialize_from_value(value).map_err(IntrisicError::InvalidRngState)?;
            Ok(Value::Null(ValueNull))
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
        Intrisic::Sum
        | Intrisic::Join
        | Intrisic::Mult
        | Intrisic::Injected(_)
        | Intrisic::SeedRNG => {
            panic!("These have no fixed param number")
        }
        Intrisic::ToJson | Intrisic::FromJson => 1,
        Intrisic::RestoreRNG => 1,
        Intrisic::SaveRNG => 0,
    }
}
