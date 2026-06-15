//! Null value

use derive_more::Display;

/// Null value
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Display)]
#[display("null")]
pub struct ValueNull;
