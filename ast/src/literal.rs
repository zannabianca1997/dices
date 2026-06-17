//! Literal values

use dices_values::{Value, bool::ValueBool, int::ValueInt, null::ValueNull, string::ValueString};

/// A literal value
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Literal {
    /// A literal null
    Null(ValueNull),
    /// A literal bool
    Bool(ValueBool),
    /// A literal int
    Int(ValueInt),
    /// A literal string
    String(ValueString),
}

impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Null(value) => value.into(),
            Literal::Bool(value) => value.into(),
            Literal::Int(value) => value.into(),
            Literal::String(value) => value.into(),
        }
    }
}
