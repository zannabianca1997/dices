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

/// 5.5. Convert
///
/// Conversions from different types, and serialization utils.
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

/// 5.5.3. ToList
///
/// Convert the argument to a list
///
/// ```dices
/// #>> let list = std.convert.list;
/// >>> list(42)
/// [42]
/// >>> list(true)
/// [true]
/// ```
///
/// Lists are left alone, while maps are converted to a list of key-value pairs,
/// sorted by keys:
///
/// ```dices
/// #>> let list = std.convert.list;
/// >>> list(<|a: 3, c: 20, b:2|>)
/// [["a", 3], ["b", 2], ["c", 20]]
/// ```
///
/// Injected values are read, and all else are enclosed in a single item list.
#[injectable]
pub fn ToList(value: Value) -> Result<ValueList, CastInjectedError> {
    ValueList::try_from(value)
}

/// 5.5.4. ToNumber
///
/// Convert the argument to an integer
///
/// ```dices
/// #>> let number = std.convert.number;
/// >>> number(true)
/// 1
/// >>> number(false)
/// 0
/// ```
#[injectable]
pub fn ToNumber(value: Value) -> Result<ValueInt, CastIntoIntError> {
    ValueInt::try_from(value)
}

/// 5.5.5. ToBool
///
/// Convert the argument to a bool. Empty containers are considered false, and
/// integers are checked against zero.
///
/// ```dices
/// #>> let bool = std.convert.bool;
/// >>> bool(null)
/// false
/// ```
///
/// ```dices
/// #>> let bool = std.convert.bool;
/// >>> bool(42)
/// true
/// >>> bool(0)
/// false
/// ```
///
/// ```dices
/// #>> let bool = std.convert.bool;
/// >>> bool([])
/// false
/// >>> bool([3])
/// true
/// ```
///
/// ```dices
/// #>> let bool = std.convert.bool;
/// >>> bool(<| |>)
/// false
/// >>> bool(<| a: 2 |>)
/// true
/// ```
#[injectable]
pub fn ToBool(value: Value) -> Result<ValueBool, CastInjectedError> {
    ValueBool::try_from(value)
}

/// 5.5.6. ToString
///
/// Convert the argument to a string
///
/// ```dices
/// #>> let string = std.convert.string;
/// >>> string(42)
/// "42"
/// >>> string([1, 2, 3])
/// "[1, 2, 3]"
/// ```
///
/// This is different from `convert.dices.serialize` as the string is human
/// readable, and infallible.
///
/// In particular strings are left unaffected instead of escaping, and injected
/// values are represented as human readable values:
/// ```dices
/// #>> let string = std.convert.string;
/// >>> string(string)
/// "<function to_string>"
/// ```
#[injectable]
pub fn ToString(value: Value) -> Result<ValueString, CastInjectedError> {
    ValueString::try_from(value)
}
