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

use dices::Dices;

pub mod dices;
pub mod json;

/// Conversion bindings
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Convert {
    pub json: Json,
    pub dices: Dices,
    pub list: ToList,
    pub number: ToNumber,
    pub bool: ToBool,
    pub string: ToString,
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
pub fn ToList(value: Value) -> Result<ValueList, CastInjectedError> {
    ValueList::try_from(value)
}

/// Convert the argument to an integer
#[injectable]
pub fn ToNumber(value: Value) -> Result<ValueInt, CastIntoIntError> {
    ValueInt::try_from(value)
}

/// Convert the argument to a bool
#[injectable]
pub fn ToBool(value: Value) -> Result<ValueBool, CastInjectedError> {
    ValueBool::try_from(value)
}

/// Convert the argument to a string
#[injectable]
pub fn ToString(value: Value) -> Result<ValueString, CastInjectedError> {
    ValueString::try_from(value)
}
