use dices_values::{Injectable, Value, injectable, string::ValueString};

/// JSON conversion bindings
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

/// Serialize a value to a JSON string
#[injectable]
pub fn Serialize(value: Value) -> Result<String, serde_json::Error> {
    serde_json::to_string(&value)
}

/// Deserialize a value from a JSON string
#[injectable]
pub fn Deserialize(text: ValueString) -> Result<Value, serde_json::Error> {
    serde_json::from_str(&text)
}
