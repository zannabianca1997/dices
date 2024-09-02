//! List of the language intrisics

use std::borrow::Cow;

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
)]
pub enum Intrisic<Injected> {
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

    /// Injected intrisic
    ///
    /// Intrisics that came from the enviroment (files, printing, exiting the shell, etc)
    Injected(Injected),
}

impl<Injected> Intrisic<Injected>
where
    Injected: InjectedIntr,
{
    /// Iter all possible intrisics
    pub fn iter() -> impl IntoIterator<Item = Self> {
        [
            Self::Sum,
            Self::Join,
            Self::Mult,
            Self::ToNumber,
            Self::ToList,
            Self::ToString,
            Self::Parse,
            Self::Call,
        ]
        .into_iter()
        .chain(Injected::iter().into_iter().map(Self::Injected))
    }

    /// Build a module containing all the intrisics, to include in the standard library
    pub fn all() -> ValueMap<Injected> {
        ValueMap::from_iter(Self::iter().into_iter().map(|v| {
            (
                v.name().to_string().into_boxed_str().into(),
                ValueIntrisic::from(v).into(),
            )
        }))
    }

    pub fn name(&self) -> Cow<str> {
        match self {
            Intrisic::Sum => "sum".into(),
            Intrisic::Join => "join".into(),
            Intrisic::Mult => "mult".into(),
            Intrisic::ToNumber => "to_number".into(),
            Intrisic::ToList => "to_list".into(),
            Intrisic::Call => "call".into(),
            Intrisic::ToString => "to_string".into(),
            Intrisic::Parse => "parse".into(),
            Intrisic::Injected(injected) => injected.name(),
        }
    }
}

pub trait InjectedIntr: Sized + Clone {
    /// The data used by the injected intrisics
    type Data;

    /// Give a name for this intrisic
    fn name(&self) -> Cow<str>;
    /// Iter all possible identifiers
    fn iter() -> impl IntoIterator<Item = Self>;
}

impl InjectedIntr for ! {
    type Data = ();

    fn name(&self) -> Cow<str> {
        *self
    }

    fn iter() -> impl IntoIterator<Item = Self> {
        []
    }
}
