#![doc = include_str!("../README.md")]

use std::{fmt::Debug, hash::Hash};

use derive_more::{Display, From, IsVariant, TryUnwrap, Unwrap};

use crate::{
    bool::ValueBool, injected::ValueInjected, int::ValueInt, list::ValueList, map::ValueMap,
    null::ValueNull, string::ValueString,
};

pub mod serde;

pub mod bool;
pub mod injected;
pub mod int;
pub mod list;
pub mod map;
pub mod null;
pub mod string;

pub mod cast;

/// A dices value
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
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
