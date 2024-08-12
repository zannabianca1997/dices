//! List of the language intrisics

use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::values::{map::ValueMap, ValueIntrisic};

#[derive(
    // display helper
    Debug,
    IntoStaticStr,
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
    /// `+`: sum two values, flattening lists,maps, and converting booleans
    #[strum(to_string = "sum")]
    Sum,
    /// `-`: subtract two values, flattening lists,maps, and converting booleans.
    ///      the second value is negated
    #[strum(to_string = "sub")]
    Sub,
    /// `-`: unary minus. Negate a value. If applied to a list, distribute over the list elements
    #[strum(to_string = "sub")]
    Neg,
    /// `~`: join two list or maps, upgrading scalars to list if joined to a list
    #[strum(to_string = "join")]
    Join,

    /// `*`: multiply two values, distributing scalars over lists or maps
    #[strum(to_string = "mult")]
    Mult,
    /// `%`: remainder of two values, distributing scalars over lists or maps
    #[strum(to_string = "rem")]
    Rem,
    /// `/`: divide two values, distributing scalars over lists or maps
    #[strum(to_string = "div")]
    Div,

    /// `d`: throw a dice, returning a value between 1 and the parameter
    #[strum(to_string = "dice")]
    Dice,

    /// Try to convert a value to a number
    #[strum(to_string = "to_number")]
    ToNumber,
    /// Try to convert a value to a list
    #[strum(to_string = "to_list")]
    ToList,

    /// Call its first parameter with the arguments given by the second, converted to a list
    #[strum(to_string = "call")]
    Call,
}

impl Intrisic {
    /// Build a module containing all the intrisics, to include in the standard library
    pub fn module() -> ValueMap {
        ValueMap::from_iter(Self::iter().map(|v| {
            (
                <&'static str>::from(v).to_string().into_boxed_str().into(),
                ValueIntrisic::from(v).into(),
            )
        }))
    }
}
