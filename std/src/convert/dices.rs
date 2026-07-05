use dices_parser::value::{ParseError, PrintError};
use dices_values::{Injectable, Value, injectable, string::ValueString};

/// 5.5.2. Dices
///
/// Dices literal serialization utils: maps simple `dices` values (not injected
/// ones) into a string that can be the mapped back.
///
/// Differently from the `json` version, this is guarantee to round-trip.
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Dices {
    pub serialize: Serialize,
    pub deserialize: Deserialize,
}

impl Dices {
    pub const fn new() -> Self {
        Self {
            serialize: Serialize,
            deserialize: Deserialize,
        }
    }
}

/// 5.5.2.1. Serialize
///
/// Serialize a value to a string with a `dices` literal
///
/// ```dices
/// #>> let dices = std.convert.dices;
/// >>> dices.serialize(42)
/// "42"
/// >>> dices.serialize([1, 2, 3])
/// "<|1, 2, 3|>"
/// >>> dices.serialize("hello")
/// "\"hello\""
/// ```
#[injectable]
pub fn Serialize(value: Value) -> Result<String, PrintError> {
    let mut s = String::new();
    dices_parser::value::print(&value, &mut s)?;
    Ok(s)
}

/// 5.5.2.2. Deserialize
///
/// Deserialize a value from a string containing a `dices` literal
///
/// ```dices
/// #>> let dices = std.convert.dices;
/// >>> dices.deserialize("42")
/// 42
/// >>> dices.deserialize("[1, 2, 3]")
/// [1, 2, 3]
/// >>> dices.deserialize("\"hello\"")
/// "hello"
/// ```
#[injectable]
pub fn Deserialize(text: ValueString) -> Result<Value, ParseError> {
    dices_parser::value::parse_value(&text)
}
