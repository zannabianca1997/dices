#![doc = include_str!("../README.md")]

use derive_more::{Display, From, IsVariant, TryInto, TryUnwrap, Unwrap};

use crate::{
    bool::ValueBool, int::ValueInt, list::ValueList, map::ValueMap, null::ValueNull, string::ValueString
};

pub mod bool;
pub mod int;
pub mod list;
pub mod null;
pub mod string;
pub mod map;

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
    TryInto,
    Unwrap,
    TryUnwrap,
    IsVariant,
)]
pub enum Value {
    Null(ValueNull),
    Bool(ValueBool),
    Int(ValueInt),
    String(ValueString),
    List(ValueList),
    Map(ValueMap)
}
