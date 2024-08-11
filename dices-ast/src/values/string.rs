use std::fmt::Display;

use derive_more::derive::{AsMut, AsRef, Deref, DerefMut, From};

use crate::fmt::quoted;

use super::{number::ValueNumber, ToNumberError};

/// An unicode string
#[derive(
    // display helper
    Debug,
    // cloning
    Clone,
    // comparisons
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    // refs
    AsRef,
    Deref,
    AsMut,
    DerefMut,
    From,
)]
pub struct ValueString(Box<str>);
impl ValueString {
    pub fn to_number(self) -> Result<ValueNumber, ToNumberError> {
        self.0.parse().map_err(ToNumberError::InvalidString)
    }
}

impl Display for ValueString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        quoted(&self.0, f)
    }
}
