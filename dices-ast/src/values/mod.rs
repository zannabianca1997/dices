//! The value a `dices` variable

use derive_more::derive::{Display, From};

use crate::intrisics::Intrisic;

pub mod bl;
pub mod closure;
pub mod list;
pub mod map;
pub mod number;
pub mod string;

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
    Bool(bl::ValueBool),
    Number(number::ValueNumber),
    String(string::ValueString),

    List(list::ValueList),
    Map(map::ValueMap),

    Intrisic(Intrisic),
    Closure(Box<closure::ValueClosure>),
}
