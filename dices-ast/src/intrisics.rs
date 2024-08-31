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
    /// `+`: sum two values, flattening lists,maps, and converting booleans
    Sum,
    /// `-`: subtract two values, flattening lists,maps, and converting booleans.
    ///      the second value is negated
    Sub,
    /// `-`: unary minus. Negate a value. If applied to a list, distribute over the list elements
    Neg,
    /// `~`: join two list or maps, upgrading scalars to list if joined to a list
    Join,

    /// `*`: multiply two values, distributing scalars over lists or maps
    Mult,
    /// `%`: remainder of two values, distributing scalars over lists or maps
    Rem,
    /// `/`: divide two values, distributing scalars over lists or maps
    Div,

    /// `d`: throw a dice, returning a value between 1 and the parameter
    Dice,

    /// Try to convert a value to a number
    ToNumber,
    /// Try to convert a value to a list
    ToList,

    /// Call its first parameter with the arguments given by the second, converted to a list
    Call,

    /// `kh`: keep the highest n values of a list or map
    KeepHigh,
    /// `kl`: keep the lowest n values of a list or map
    KeepLow,
    /// `rh`: keep the highest n values of a list or map
    RemoveHigh,
    /// `rl`: keep the lowest n values of a list or map
    RemoveLow,
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
            Intrisic::Sub => "sub",
            Intrisic::Neg => "sub",
            Intrisic::Join => "join",
            Intrisic::Mult => "mult",
            Intrisic::Rem => "rem",
            Intrisic::Div => "div",
            Intrisic::Dice => "dice",
            Intrisic::ToNumber => "to_number",
            Intrisic::ToList => "to_list",
            Intrisic::Call => "call",
            Intrisic::KeepHigh => "keep_high",
            Intrisic::KeepLow => "keep_low",
            Intrisic::RemoveHigh => "remove_high",
            Intrisic::RemoveLow => "remove_low",
        }
    }
}
