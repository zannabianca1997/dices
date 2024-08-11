//! The value a `dices` variable

use std::num::ParseIntError;

use derive_more::derive::{Display, Error, From};

use crate::intrisics::Intrisic;

pub mod bl;
pub mod closure;
pub mod list;
pub mod map;
pub mod number;
pub mod string;

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
pub enum Value {
    Bool(bl::ValueBool),
    Number(number::ValueNumber),
    String(string::ValueString),

    List(list::ValueList),
    Map(map::ValueMap),

    Intrisic(Intrisic),
    Closure(Box<closure::ValueClosure>),
}

impl Value {
    pub fn to_number(self) -> Result<number::ValueNumber, ToNumberError> {
        match self {
            Value::Bool(v) => v.to_number(),
            Value::Number(v) => v.to_number(),
            Value::String(v) => v.to_number(),
            Value::List(v) => v.to_number(),
            Value::Map(v) => v.to_number(),
            Value::Intrisic(v) => v.to_number(),
            Value::Closure(v) => v.to_number(),
        }
    }
}

#[derive(Debug, Display, Error)]
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
}
