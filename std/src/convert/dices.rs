use dices_parser::value::{ParseError, PrintError};
use dices_values::{Injectable, Value, injectable, string::ValueString};

/// JSON conversion bindings
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Dices {
    serialize: Serialize,
    deserialize: Deserialize,
}

impl Dices {
    pub const fn new() -> Self {
        Self {
            serialize: Serialize,
            deserialize: Deserialize,
        }
    }
}

/// Serialize a value to a string with a dices literal
#[injectable]
fn Serialize(value: Value) -> Result<String, PrintError> {
    let mut s = String::new();
    dices_parser::value::print(&value, &mut s)?;
    Ok(s)
}

/// Deserialize a value to a string containing a dices
#[injectable]
fn Deserialize(text: ValueString) -> Result<Value, ParseError> {
    dices_parser::value::parse_value(&text)
}
