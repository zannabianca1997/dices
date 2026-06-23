use dices_values::{
    Injectable, Value,
    bool::ValueBool,
    cast::{CastInjectedError, CastIntoIntError},
    injectable,
    int::ValueInt,
    list::ValueList,
    string::ValueString,
};
use json::Json;

use crate::convert::dices::Dices;

mod dices;
mod json;

/// Conversion bindings
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Convert {
    json: Json,
    dices: Dices,
    list: ToList,
    number: ToNumber,
    bool: ToBool,
    string: ToString,
}

impl Convert {
    pub const fn new() -> Self {
        Self {
            json: Json::new(),
            dices: Dices::new(),
            list: ToList,
            number: ToNumber,
            bool: ToBool,
            string: ToString,
        }
    }
}

/// Convert the argument to a list
#[injectable]
fn ToList(value: Value) -> Result<ValueList, CastInjectedError> {
    ValueList::try_from(value)
}

/// Convert the argument to an integer
#[injectable]
fn ToNumber(value: Value) -> Result<ValueInt, CastIntoIntError> {
    ValueInt::try_from(value)
}

/// Convert the argument to a bool
#[injectable]
fn ToBool(value: Value) -> Result<ValueBool, CastInjectedError> {
    ValueBool::try_from(value)
}

/// Convert the argument to a string
#[injectable]
fn ToString(value: Value) -> Result<ValueString, CastInjectedError> {
    ValueString::try_from(value)
}
