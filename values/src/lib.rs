#![doc = include_str!("../README.md")]

use std::{fmt::Debug, hash::Hash};

use derive_more::{Display, From, IsVariant, TryUnwrap, Unwrap};

use crate::{
    bool::ValueBool, injected::ValueInjected, int::ValueInt, list::ValueList, map::ValueMap,
    null::ValueNull, string::ValueString,
};

pub mod pretty;
pub mod serde;

pub mod bool;
pub mod identifier;
pub mod injected;
pub mod int;
pub mod list;
pub mod map;
pub mod null;
pub mod string;

pub mod cast;
pub mod utils;

#[cfg(feature = "macros")]
pub use dices_values_macros::{Injectable, injectable};

/// A dices value
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Display,
    From,
    Unwrap,
    TryUnwrap,
    IsVariant,
    strum::EnumDiscriminants,
)]
#[strum_discriminants(
    name(Type),
    derive(strum::Display, Hash),
    doc = "Types of a [`Value`]",
    vis(pub)
)]
#[unwrap(ref)]
pub enum Value {
    Null(ValueNull),
    Bool(ValueBool),
    Int(ValueInt),
    String(ValueString),
    List(ValueList),
    Map(ValueMap),
    Injected(ValueInjected),
}

impl Value {
    pub fn typ(&self) -> Type {
        self.into()
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Null(ValueNull)
    }
}
