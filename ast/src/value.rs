//! The value a `dices` variable

use derive_more::derive::{Display, From, TryUnwrap, Unwrap};

pub use boolean::ValueBool;
pub use closure::ValueClosure;
pub use intrisics::ValueIntrisic;
pub use list::ValueList;
pub use map::ValueMap;
pub use null::ValueNull;
pub use number::ValueNumber;
pub use string::ValueString;
use thiserror::Error;

use crate::intrisics::{Intrisic, NoInjectedIntrisics};

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

#[cfg(feature = "parse_value")]
mod parse;

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
    // Members
    Unwrap,
    TryUnwrap,
    enum_as_inner::EnumAsInner,
    // conversion
    From,
)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode,),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
#[from(forward)]
pub enum Value<InjectedIntrisic = NoInjectedIntrisics> {
    Null(ValueNull),
    Bool(ValueBool),
    Number(ValueNumber),
    String(ValueString),

    List(ValueList<InjectedIntrisic>),
    Map(ValueMap<InjectedIntrisic>),

    Intrisic(ValueIntrisic<InjectedIntrisic>),
    Closure(Box<ValueClosure<InjectedIntrisic>>),
}

impl Value<NoInjectedIntrisics> {
    // Add any intrisic type to a intrisic-less value
    pub fn with_arbitrary_injected_intrisics<II>(self) -> Value<II> {
        match self {
            Value::Null(value_null) => Value::Null(value_null),
            Value::Bool(value_bool) => Value::Bool(value_bool),
            Value::Number(value_number) => Value::Number(value_number),
            Value::String(value_string) => Value::String(value_string),
            Value::List(value_list) => Value::List(value_list.with_arbitrary_injected_intrisics()),
            Value::Map(value_map) => Value::Map(value_map.with_arbitrary_injected_intrisics()),
            Value::Intrisic(value_intrisic) => {
                Value::Intrisic(value_intrisic.with_arbitrary_injected_intrisics())
            }
            Value::Closure(value_closure) => {
                Value::Closure(Box::new(value_closure.with_arbitrary_injected_intrisics()))
            }
        }
    }
}

impl<InjectedIntrisic> Value<InjectedIntrisic> {
    #[cfg(feature = "parse_value")]
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

#[cfg(feature = "pretty")]
impl<'a, D, A, II> pretty::Pretty<'a, D, A> for &'a Value<II>
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
    II: crate::intrisics::InjectedIntr,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        match self {
            Value::Null(value) => value.pretty(allocator),
            Value::Bool(value) => value.pretty(allocator),
            Value::Number(value) => value.pretty(allocator),
            Value::String(value) => value.pretty(allocator),
            Value::List(value) => value.pretty(allocator),
            Value::Map(value) => value.pretty(allocator),
            Value::Intrisic(value) => value.pretty(allocator),
            Value::Closure(value) => value.pretty(allocator),
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum ToNumberError {
    #[cfg(feature = "parse_value")]
    #[error("The string cannot be converted in a number")]
    InvalidString(#[source] <Value as std::str::FromStr>::Err),
    #[error("A list of length {_0} cannot be interpreted as a number")]
    WrongListLength(usize),
    #[error("A map of length {_0} cannot be interpreted as a number")]
    WrongMapLength(usize),
    #[error("Intrisics cannot be interpreted as a number")]
    Intrisic,
    #[error("Closures cannot be interpreted as a number")]
    Closure,
    #[error("`null` cannot be interpreted as a number")]
    InvalidNull,
}

#[derive(Debug, Error, Clone)]
pub enum ToListError {}

impl<InjectedIntrisic> From<Intrisic<InjectedIntrisic>> for Value<InjectedIntrisic> {
    fn from(value: Intrisic<InjectedIntrisic>) -> Self {
        Value::Intrisic(value.into())
    }
}

#[cfg(feature = "serde")]
pub mod serde;
