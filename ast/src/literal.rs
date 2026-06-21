//! Literal values

use derive_more::{From, IsVariant, TryInto, TryUnwrap, Unwrap};
use dices_values::Value;

mod boolean;
mod integer;
mod null;
mod string;

pub use boolean::LiteralBool;
pub use integer::LiteralInt;
pub use null::LiteralNull;
pub use string::LiteralString;

/// A literal value
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IsVariant, TryUnwrap, From, TryInto, Unwrap,
)]
pub enum Literal {
    /// A literal null
    Null(LiteralNull),
    /// A literal bool
    Bool(LiteralBool),
    /// A literal int
    Int(LiteralInt),
    /// A literal string
    String(LiteralString),
}

impl From<Literal> for Value {
    fn from(value: Literal) -> Self {
        match value {
            Literal::Null(value) => value.0.into(),
            Literal::Bool(value) => value.0.into(),
            Literal::Int(value) => value.0.into(),
            Literal::String(value) => value.0.into(),
        }
    }
}
