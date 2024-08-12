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

pub mod boolean;
pub mod closure;
pub mod list;
pub mod map;
pub mod null;
pub mod number;
pub mod string;
pub mod intrisics {

    use std::fmt::Display;

    use derive_more::derive::From;

    use crate::intrisics::Intrisic;

    use super::{ToListError, ToNumberError, ValueList, ValueNumber};

    /// Value representing an intrisic
    #[derive(
        // display helper
        Debug,
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
    pub struct ValueIntrisic(Intrisic);

    impl Display for ValueIntrisic {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "<intrisic {}>", <&'static str>::from(self.0))
        }
    }

    impl ValueIntrisic {
        pub fn to_number(&self) -> Result<ValueNumber, ToNumberError> {
            Err(ToNumberError::Intrisic)
        }
        pub fn to_list(self) -> Result<ValueList, ToListError> {
            Ok(ValueList::from_iter([self.into()]))
        }
    }
}

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
    Null(ValueNull),
    Bool(ValueBool),
    Number(ValueNumber),
    String(ValueString),

    List(ValueList),
    Map(ValueMap),

    Intrisic(ValueIntrisic),
    Closure(Box<ValueClosure>),
}

impl Value {
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

    pub fn to_list(self) -> Result<ValueList, ToListError> {
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
    InvalidNull,
}

#[derive(Debug, Display, Error)]
pub enum ToListError {}

#[cfg(test)]
impl<'a> arbitrary::Arbitrary<'a> for Value {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match u.choose_index(6)? {
            0 => ValueNull::arbitrary(u)?.into(),
            1 => ValueBool::arbitrary(u)?.into(),
            2 => ValueNumber::arbitrary(u)?.into(),
            3 => ValueString::arbitrary(u)?.into(),
            4 => ValueList::arbitrary(u)?.into(),
            5 => ValueMap::arbitrary(u)?.into(),
            _ => unreachable!(),
        })
    }
    fn arbitrary_take_rest(mut u: arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match u.choose_index(6)? {
            0 => ValueNull::arbitrary_take_rest(u)?.into(),
            1 => ValueBool::arbitrary_take_rest(u)?.into(),
            2 => ValueNumber::arbitrary_take_rest(u)?.into(),
            3 => ValueString::arbitrary_take_rest(u)?.into(),
            4 => ValueList::arbitrary_take_rest(u)?.into(),
            5 => ValueMap::arbitrary_take_rest(u)?.into(),
            _ => unreachable!(),
        })
    }

    fn size_hint(depth: usize) -> (usize, Option<usize>) {
        use arbitrary::size_hint;

        size_hint::or_all(&[
            ValueNull::size_hint(depth),
            ValueBool::size_hint(depth),
            ValueNumber::size_hint(depth),
            ValueString::size_hint(depth),
            // those two variants might be recursive, and need to be guarded
            size_hint::recursion_guard(depth, |depth| {
                size_hint::or_all(&[ValueList::size_hint(depth), ValueMap::size_hint(depth)])
            }),
        ])
    }
}
