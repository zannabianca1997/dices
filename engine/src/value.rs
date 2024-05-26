use std::{collections::HashMap, mem, rc::Rc};

use strum::{EnumDiscriminants, EnumIs, EnumTryAs, IntoStaticStr};
use thiserror::Error;

use crate::{
    expr::{EvalError, Expr},
    identifier::IdentStr,
};

/// String type in `dices`
pub type DString = Rc<str>;

#[derive(Debug, Clone, EnumDiscriminants, EnumTryAs, PartialEq, Eq, Default)]
#[strum_discriminants(name(Type), derive(EnumIs, IntoStaticStr, strum::Display))]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(tag = "type", content = "value", deny_unknown_fields)
)]
/// Value of a variable in `dices`
pub enum Value {
    /// A callable
    Function {
        /// Parameters of the function
        params: Rc<[Rc<IdentStr>]>,
        /// Body of the function
        body: Rc<Expr>,
    },

    // Plain data types
    /// Null
    #[default]
    Null,
    /// Boolean
    Bool(bool),
    /// Number
    Number(i64),
    /// List
    List(Vec<Self>),
    /// String
    String(DString),
    /// Map
    Map(HashMap<DString, Self>),
}

impl Value {
    pub fn to_number(&self) -> Result<i64, ToNumberError> {
        Ok(match self {
            Value::Bool(b) => match b {
                true => 1,
                false => 0,
            },
            Value::Number(n) => *n,
            Value::List(l) => {
                if let [n] = &**l {
                    n.to_number()?
                } else {
                    return Err(ToNumberError::ListNotSingular(l.len()));
                }
            }

            _ => return Err(ToNumberError::InvalidType(self.type_())),
        })
    }

    pub fn try_to_number(self) -> Result<i64, Self> {
        self.to_number().map_err(|_| self)
    }

    pub fn type_(&self) -> Type {
        Type::from(self)
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ToNumberError {
    #[error("List of length {0} is not a valid number")]
    ListNotSingular(usize),
    #[error("Type {0} is not acceptable as a number")]
    InvalidType(Type),
}

pub fn sum(a: Value) -> Result<i64, EvalError> {
    match a.try_to_number() {
        Ok(n) => Ok(n),
        Err(Value::List(l)) => l.into_iter().map(sum).try_fold(0i64, |a, b| {
            b.and_then(|b| a.checked_add(b).ok_or(EvalError::IntegerOverflow))
        }),
        Err(v) => Err(EvalError::InvalidType("+", v.type_())),
    }
}

pub fn neg(a: Value) -> Result<Value, EvalError> {
    match a.try_to_number() {
        Ok(n) => n
            .checked_neg()
            .map(Value::Number)
            .ok_or(EvalError::IntegerOverflow),
        Err(Value::List(mut l)) => {
            for v in l.iter_mut() {
                let v_ = mem::take(v);
                *v = neg(v_)?
            }
            Ok(Value::List(l))
        }
        Err(err) => Err(EvalError::InvalidType("-", err.type_())),
    }
}

pub fn mul(a: Value, b: Value) -> Result<Value, EvalError> {
    Ok(match (a.try_to_number(), b.try_to_number()) {
        (Ok(a), Ok(b)) => Value::Number(a.checked_mul(b).ok_or(EvalError::IntegerOverflow)?),
        (Ok(n), Err(Value::List(l))) | (Err(Value::List(l)), Ok(n)) => Value::List(
            l.into_iter()
                .map(|v| {
                    v.to_number()?
                        .checked_mul(n)
                        .map(Value::Number)
                        .ok_or(EvalError::IntegerOverflow)
                })
                .try_collect()?,
        ),
        (Ok(_), Err(v)) => return Err(EvalError::InvalidTypes("*", Type::Number, v.type_())),
        (Err(v), Ok(_)) => return Err(EvalError::InvalidTypes("*", v.type_(), Type::Number)),
        (Err(v1), Err(v2)) => return Err(EvalError::InvalidTypes("*", v1.type_(), v2.type_())),
    })
}

pub fn div(a: Value, b: Value) -> Result<Value, EvalError> {
    Ok(match (a.try_to_number(), b.try_to_number()) {
        (Ok(a), Ok(b)) => Value::Number(a.checked_div(b).ok_or(EvalError::IntegerOverflow)?),
        (Ok(n), Err(Value::List(l))) => Value::List(
            l.into_iter()
                .map(|v| {
                    n.checked_div(v.to_number()?)
                        .map(Value::Number)
                        .ok_or(EvalError::IntegerOverflow)
                })
                .try_collect()?,
        ),
        (Err(Value::List(l)), Ok(n)) => Value::List(
            l.into_iter()
                .map(|v| {
                    v.to_number()?
                        .checked_div(n)
                        .map(Value::Number)
                        .ok_or(EvalError::IntegerOverflow)
                })
                .try_collect()?,
        ),
        (Ok(_), Err(v)) => return Err(EvalError::InvalidTypes("/", Type::Number, v.type_())),
        (Err(v), Ok(_)) => return Err(EvalError::InvalidTypes("/", v.type_(), Type::Number)),
        (Err(v1), Err(v2)) => return Err(EvalError::InvalidTypes("/", v1.type_(), v2.type_())),
    })
}

pub fn rem(a: Value, b: Value) -> Result<Value, EvalError> {
    Ok(match (a.try_to_number(), b.try_to_number()) {
        (Ok(a), Ok(b)) => Value::Number(a.checked_rem(b).ok_or(EvalError::IntegerOverflow)?),
        (Ok(n), Err(Value::List(l))) => Value::List(
            l.into_iter()
                .map(|v| {
                    n.checked_rem(v.to_number()?)
                        .map(Value::Number)
                        .ok_or(EvalError::IntegerOverflow)
                })
                .try_collect()?,
        ),
        (Err(Value::List(l)), Ok(n)) => Value::List(
            l.into_iter()
                .map(|v| {
                    v.to_number()?
                        .checked_rem(n)
                        .map(Value::Number)
                        .ok_or(EvalError::IntegerOverflow)
                })
                .try_collect()?,
        ),
        (Ok(_), Err(v)) => return Err(EvalError::InvalidTypes("%", Type::Number, v.type_())),
        (Err(v), Ok(_)) => return Err(EvalError::InvalidTypes("%", v.type_(), Type::Number)),
        (Err(v1), Err(v2)) => return Err(EvalError::InvalidTypes("%", v1.type_(), v2.type_())),
    })
}

pub fn join(a: Value, b: Value) -> Value {
    match (a, b) {
        (Value::List(mut l1), Value::List(mut l2)) => {
            l1.append(&mut l2);
            Value::List(l1)
        }
        (Value::String(s1), Value::String(s2)) => {
            let mut s = String::with_capacity(s1.len() + s2.len());
            s.push_str(&*s1);
            s.push_str(&*s2);
            Value::String(s.into())
        }
        (Value::Map(mut m1), Value::Map(m2)) => {
            m1.extend(m2);
            Value::Map(m1)
        }

        // upgrading to lists
        (a @ Value::List(_), b) => join(a, Value::List(vec![b])),
        (a, b) => join(Value::List(vec![a]), b),
    }
}
