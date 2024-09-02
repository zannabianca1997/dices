//! The value a `dices` variable

use std::num::ParseIntError;

use derive_more::derive::{Display, Error, From};

pub use boolean::ValueBool;
pub use closure::ValueClosure;
pub use intrisics::ValueIntrisic;
pub use list::ValueList;
pub use map::ValueMap;
pub use null::ValueNull;
pub use number::ValueNumber;
pub use string::ValueString;

use crate::intrisics::Intrisic;

pub mod boolean;
pub mod closure;
pub mod intrisics;
pub mod list;
pub mod map;
pub mod null;
pub mod number;
pub mod string;

#[cfg(test)]
mod tests;

#[derive(
    // display helper
    Debug,
    Display,
    // cloning
    Clone,
    // comparisons
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    // conversion
    From,
)]
pub enum Value<InjectedIntrisic> {
    Null(ValueNull),
    Bool(ValueBool),
    Number(ValueNumber),
    String(ValueString),

    List(ValueList<InjectedIntrisic>),
    Map(ValueMap<InjectedIntrisic>),

    Intrisic(ValueIntrisic<InjectedIntrisic>),
    Closure(Box<ValueClosure<InjectedIntrisic>>),
}

impl<InjectedIntrisic> Value<InjectedIntrisic> {
    pub fn to_number(self) -> Result<ValueNumber, ToNumberError> {
        match self {
            Value::Bool(v) => v.to_number(),
            Value::Number(v) => v.to_number(),
            Value::String(v) => v.to_number(),
            Value::List(v) => v.to_number(),
            Value::Map(v) => v.to_number(),
            Value::Intrisic(v) => v.to_number(),
            Value::Closure(v) => v.to_number(),
            Value::Null(v) => v.to_number(),
        }
    }

    pub fn to_list(self) -> Result<ValueList<InjectedIntrisic>, ToListError> {
        match self {
            Value::Bool(v) => v.to_list(),
            Value::Number(v) => v.to_list(),
            Value::String(v) => v.to_list(),
            Value::List(v) => v.to_list(),
            Value::Map(v) => v.to_list(),
            Value::Intrisic(v) => v.to_list(),
            Value::Closure(v) => v.to_list(),
            Value::Null(v) => v.to_list(),
        }
    }
}

#[derive(Debug, Display, Error, Clone)]
pub enum ToNumberError {
    #[display("The string cannot be converted in a number")]
    InvalidString(#[error(source)] ParseIntError),
    #[display("A list of length {} cannot be interpreted as a number", 0)]
    WrongListLength(#[error(not(source))] usize),
    #[display("A map of length {} cannot be interpreted as a number", 0)]
    WrongMapLength(#[error(not(source))] usize),
    #[display("Intrisics cannot be interpreted as a number")]
    Intrisic,
    #[display("Closures cannot be interpreted as a number")]
    Closure,
    #[display("`null` cannot be interpreted as a number")]
    InvalidNull,
}

#[derive(Debug, Display, Error, Clone)]
pub enum ToListError {}

impl<InjectedIntrisic> From<Intrisic<InjectedIntrisic>> for Value<InjectedIntrisic> {
    fn from(value: Intrisic<InjectedIntrisic>) -> Self {
        Value::Intrisic(value.into())
    }
}
