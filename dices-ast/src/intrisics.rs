//! List of the language intrisics

use std::{
    borrow::Cow,
    error::Error,
    fmt::{Debug, Display},
    hash::Hash,
};

use crate::values::{map::ValueMap, Value, ValueIntrisic};

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
    /// The error type given by calling this intrisic
    type Error: Error + Clone + 'static;

    /// Iter all possible intrisics that must be injected
    fn iter() -> impl IntoIterator<Item = Self>;
    /// Give a name for this intrisic
    fn name(&self) -> Cow<str>;
    /// Call this intrisic
    fn call(
        &self,
        data: &mut Self::Data,
        params: Box<[Value<Self>]>,
    ) -> Result<Value<Self>, Self::Error>;
}

/// No injected intrisics
#[derive(Clone, Copy)]
pub struct NoInjectedIntrisics(!);

impl Debug for NoInjectedIntrisics {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
    }
}
impl Display for NoInjectedIntrisics {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0
    }
}
impl PartialEq for NoInjectedIntrisics {
    fn eq(&self, _: &Self) -> bool {
        self.0
    }
}
impl Eq for NoInjectedIntrisics {}
impl PartialOrd for NoInjectedIntrisics {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        self.0
    }
}
impl Ord for NoInjectedIntrisics {
    fn cmp(&self, _: &Self) -> std::cmp::Ordering {
        self.0
    }
}
impl Hash for NoInjectedIntrisics {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {
        self.0
    }
}

impl InjectedIntr for NoInjectedIntrisics {
    type Data = ();
    type Error = !;

    fn iter() -> impl IntoIterator<Item = Self> {
        []
    }

    fn name(&self) -> Cow<str> {
        self.0
    }

    fn call(&self, _: &mut Self::Data, _: Box<[Value<Self>]>) -> Result<Value<Self>, Self::Error> {
        self.0
    }
}
