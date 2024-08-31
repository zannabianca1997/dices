use std::{borrow::Borrow, fmt::Display};

use derive_more::derive::{AsMut, AsRef, Deref, DerefMut, From, Into};

use crate::fmt::quoted;

use super::{list::ValueList, number::ValueNumber, ToNumberError};

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
    // conversions
    From,
    Into,
)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct ValueString(Box<str>);
impl ValueString {
    pub fn to_number(self) -> Result<ValueNumber, ToNumberError> {
        self.0
            .parse::<i64>()
            .map(Into::into)
            .map_err(ToNumberError::InvalidString)
    }
    pub fn to_list(self) -> Result<ValueList, super::ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}

impl From<&str> for ValueString {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<String> for ValueString {
    fn from(value: String) -> Self {
        Self(value.into())
    }
}

impl Borrow<str> for ValueString {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Display for ValueString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        quoted(&self.0, f)
    }
}
