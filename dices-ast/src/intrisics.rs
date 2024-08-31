//! List of the language intrisics

use strum::{EnumIter, IntoEnumIterator};

use crate::values::{map::ValueMap, ValueIntrisic};

#[derive(
    // display helper
    Debug,
    // cloning
    Clone,
    Copy,
    // comparisons
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    // iter all
    EnumIter,
)]
pub enum Intrisic {
    /// `+`: sum multiple values, flattening lists,maps, and converting booleans
    Sum,
    /// `~`: join multiple list or maps, upgrading scalars to list if joined to a list
    Join,

    /// `*`: multiply multiple values, distributing scalars over lists or maps
    Mult,

    /// Try to convert a value to a number
    ToNumber,
    /// Try to convert a value to a list
    ToList,
    /// Convert a value to a string
    ToString,
    /// Parse a string into a value
    Parse,

    /// Call its first parameter with the arguments given by the second, converted to a list
    Call,
}

impl Intrisic {
    /// Build a module containing all the intrisics, to include in the standard library
    pub fn all() -> ValueMap {
        ValueMap::from_iter(Self::iter().map(|v| {
            (
                v.name().to_string().into_boxed_str().into(),
                ValueIntrisic::from(v).into(),
            )
        }))
    }

    pub fn name(self) -> &'static str {
        match self {
            Intrisic::Sum => "sum",
            Intrisic::Join => "join",
            Intrisic::Mult => "mult",
            Intrisic::ToNumber => "to_number",
            Intrisic::ToList => "to_list",
            Intrisic::Call => "call",
            Intrisic::ToString => "to_string",
            Intrisic::Parse => "parse",
        }
    }
}
