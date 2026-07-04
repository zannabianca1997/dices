use dices_values::{Injectable, Value, injectable, string::ValueString};

/// 5.5.1. Json
///
/// Json literal serialization utils: maps simple `dices` values (not injected
/// ones) into JSON strings that can be the mapped back.
///
/// The conversion is not perfect: in particular only integer values in the
/// range (-2^127, 2^128) are for now correctly handled.
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Json {
    pub serialize: Serialize,
    pub deserialize: Deserialize,
}

impl Json {
    pub const fn new() -> Self {
        Self {
            serialize: Serialize,
            deserialize: Deserialize,
        }
    }
}

/// 5.5.1.1. Serialize
///
/// Serialize a value to a JSON string
///
/// ```dices
/// #>> let json = std.convert.json;
/// >>> json.serialize([1, 2, 3])
/// "[1,2,3]"
/// >>> json.serialize("hello")
/// "\"hello\""
/// >>> json.serialize(null)
/// "null"
/// ```
#[injectable]
pub fn Serialize(value: Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(&value)
}

/// 5.5.1.2. Deserialize
///
/// Deserialize a value from a JSON string
///
/// ```dices
/// #>> let json = std.convert.json;
/// >>> json.deserialize("[1, 2, 3]")
/// [1, 2, 3]
/// >>> json.deserialize("\"hello\"")
/// "hello"
/// >>> json.deserialize("null")
/// null
/// ```
#[injectable]
pub fn Deserialize(text: ValueString) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&text)
}
