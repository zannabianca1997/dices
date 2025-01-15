//! Intrisic operations

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
    str::FromStr,
};

use rand::SeedableRng;
use thiserror::Error;

use dices_ast::{
    expression::{bin_ops::BinOp, Expression, ExpressionBinOp, ExpressionCall},
    intrisics::{InjectedIntr, Intrisic},
    value::{
        serde::{deserialize_from_value, serialize_to_value},
        ToListError, ToNumberError, Value, ValueIntrisic, ValueList, ValueNull,
    },
};

use crate::{solve::Solvable, DicesRng};

use super::SolveError;

#[derive(Debug, Error)]
pub enum IntrisicError<Injected>
where
    Injected: InjectedIntr,
{
    #[error("Wrong number of params given to the intrisic {}: expected {}, given {given}", called.name(), param_num(called).unwrap())]
    WrongParamNum {
        called: Intrisic<Injected>,
        given: usize,
    },
    #[error("Expression called failed to evaluate")]
    CallFailed(#[source] SolveError<Injected>),
    #[error("Error during summing")]
    SumFailed(#[source] SolveError<Injected>),
    #[error("Error during multiplying")]
    MultFailed(#[source] SolveError<Injected>),
    #[error("Error during joining")]
    JoinFailed(#[source] SolveError<Injected>),
    #[error("The second parameter of `call` must be a list of parameters")]
    CallParamsNotAList(#[source] ToListError),
    #[error("Cannot convert to a number")]
    ToNumber(#[source] ToNumberError),
    #[error("Cannot convert to a list")]
    ToList(#[source] ToListError),
    #[error("`parse` must be called on a string, not on {_0}")]
    CannotParseNonString(Value<Injected>),
    #[error("`from_json` must be called on a string, not on {_0}")]
    JsonMustBeString(Value<Injected>),
    #[error("Failed to parse string")]
    ParseFailed(#[source] <Value<Injected> as FromStr>::Err),
    #[error(transparent)]
    Injected(Injected::Error),
    #[error("Cannot deserialize from json")]
    JsonError(#[source] Rc<serde_json::Error>),
    #[error("Invalid RNG state")]
    InvalidRngState(#[source] dices_ast::value::serde::DeserializeFromValueError),
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
                Ok(ab) => *ab,
                Err(s) => {
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
        Intrisic::Sum => {
            if params.is_empty() {
                return Ok(Value::Number(0.into()));
            }
            let mut params = params.into_vec().into_iter();
            let mut acc = params.next().unwrap();
            for p in params {
                acc = Expression::BinOp(ExpressionBinOp::new(BinOp::Add, acc.into(), p.into()))
                    .solve(context)
                    .map_err(IntrisicError::SumFailed)?;
            }
            Ok(acc)
        }
        Intrisic::Join => {
            if params.is_empty() {
                return Ok(Value::List(ValueList::new()));
            }
            let mut params = params.into_vec().into_iter();
            let mut acc = params.next().unwrap();
            for p in params {
                acc = Expression::BinOp(ExpressionBinOp::new(BinOp::Join, acc.into(), p.into()))
                    .solve(context)
                    .map_err(IntrisicError::JoinFailed)?;
            }
            Ok(acc)
        }
        Intrisic::Mult => {
            if params.is_empty() {
                return Ok(Value::Number(1.into()));
            }
            let mut params = params.into_vec().into_iter();
            let mut acc = params.next().unwrap();
            for p in params {
                acc = Expression::BinOp(ExpressionBinOp::new(BinOp::Mult, acc.into(), p.into()))
                    .solve(context)
                    .map_err(IntrisicError::MultFailed)?;
            }
            Ok(acc)
        }

        // Conversions
        Intrisic::ToNumber => {
            let [value] = match Box::<[_; 1]>::try_from(params) {
                Ok(a) => *a,
                Err(s) => {
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
                Ok(a) => *a,
                Err(s) => {
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
                Ok(a) => *a,
                Err(s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::ToString,
                        given: s.len(),
                    })
                }
            };
            Ok(Value::String(value.to_string().into()))
        }
        Intrisic::Parse => {
            let value = match Box::<[_; 1]>::try_from(params) {
                Ok(s) => match *s {
                    [Value::String(value_string)] => value_string,
                    [a] => return Err(IntrisicError::CannotParseNonString(a)),
                },
                Err(s) => {
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
                Ok(v) => *v,
                Err(s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::ToJson,
                        given: s.len(),
                    })
                }
            };
            serde_json::to_string(&value)
                .map(|s| Value::String(s.into()))
                .map_err(|err| IntrisicError::JsonError(Rc::new(err)))
        }
        Intrisic::FromJson => {
            let value = match Box::<[_; 1]>::try_from(params) {
                Ok(s) => match *s {
                    [Value::String(value_string)] => value_string,
                    [a] => return Err(IntrisicError::JsonMustBeString(a)),
                },
                Err(s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::FromJson,
                        given: s.len(),
                    })
                }
            };
            serde_json::from_str(&value).map_err(|err| IntrisicError::JsonError(Rc::new(err)))
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
                Ok(v) => *v,
                Err(s) => {
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

const fn param_num<Injected>(intr: &Intrisic<Injected>) -> Option<usize> {
    match intr {
        Intrisic::Call => Some(2),
        Intrisic::ToString
        | Intrisic::Parse
        | Intrisic::ToNumber
        | Intrisic::ToList
        | Intrisic::ToJson
        | Intrisic::FromJson
        | Intrisic::RestoreRNG => Some(1),
        Intrisic::SaveRNG => Some(0),
        Intrisic::Sum
        | Intrisic::Join
        | Intrisic::Mult
        | Intrisic::Injected(_)
        | Intrisic::SeedRNG => None,
    }
}
