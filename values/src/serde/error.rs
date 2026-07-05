//! Errors produced while (de)serializing a dices [`Value`].

use std::fmt::Display;

use serde::{de, ser};
use snafu::Snafu;

use crate::{Type, Value, int::ValueInt, string::ValueString};

/// An error raised while mapping a dices [`Value`] to or from a Rust type.
#[derive(Debug, Clone, PartialEq, Eq, Snafu)]
#[snafu(visibility(pub(super)))]
pub enum Error {
    /// A free-form message produced by `serde` (via [`de::Error::custom`] or
    /// [`ser::Error::custom`]).
    #[snafu(display("{message}"))]
    Custom {
        /// The message provided by `serde`.
        message: String,
    },
    /// A value of a different type than the one requested was found.
    #[snafu(display("expected {expected}, found {found}"))]
    UnexpectedType {
        /// The kind of value the type expected.
        expected: &'static str,
        /// The kind of value actually present.
        found: Type,
    },
    /// An integer was present but did not fit in the requested Rust type.
    #[snafu(display("integer {found} is out of range for the requested type"))]
    IntegerOutOfRange {
        /// The integer that could not be represented.
        found: ValueInt,
    },
    /// A `char` was requested but the string did not hold exactly one.
    #[snafu(display("expected a single character, found {found:?}"))]
    InvalidChar {
        /// The offending string.
        found: ValueString,
    },
    /// Floating point numbers have no representation as a dices value.
    #[snafu(display("floating point numbers cannot be represented as a dices value"))]
    FloatUnsupported,
    /// A map key could not be represented as a dices string key.
    #[snafu(display("a {found} cannot be used as a map key"))]
    InvalidMapKey {
        /// The kind of value that was offered as a key.
        found: &'static str,
    },
    /// A value was not a valid externally-tagged enum representation.
    #[snafu(display("invalid enum representation: {reason}"))]
    InvalidEnum {
        /// What was wrong with the representation.
        reason: &'static str,
    },
}

impl Error {
    /// Build an [`Error::UnexpectedType`] from the value that was found.
    pub(crate) fn unexpected(expected: &'static str, found: &Value) -> Self {
        Error::UnexpectedType {
            expected,
            found: Type::from(found),
        }
    }
}

impl de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom {
            message: msg.to_string(),
        }
    }
}

impl ser::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom {
            message: msg.to_string(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
