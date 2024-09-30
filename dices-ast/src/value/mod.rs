//! The value a `dices` variable

use derive_more::derive::{Display, Error, From, TryUnwrap, Unwrap};

pub use boolean::ValueBool;
pub use closure::ValueClosure;
pub use intrisics::ValueIntrisic;
pub use list::ValueList;
pub use map::ValueMap;
pub use null::ValueNull;
pub use number::ValueNumber;
pub use string::ValueString;

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
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr + 'static")
)]
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

#[derive(Debug, Display, Error, Clone)]
pub enum ToNumberError {
    #[cfg(feature = "parse_value")]
    #[display("The string cannot be converted in a number")]
    InvalidString(#[error(source)] <Value as std::str::FromStr>::Err),
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
