use std::{borrow::Borrow, fmt::Display};

use derive_more::derive::{AsMut, AsRef, Deref, DerefMut, From, Into};

use crate::fmt::quoted;

use super::list::ValueList;

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
#[cfg_attr(feature = "bincode", derive(bincode::Decode, bincode::Encode,))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValueString(Box<str>);
impl ValueString {
    #[cfg(feature = "parse_value")]
    pub fn to_number(self) -> Result<super::ValueNumber, super::ToNumberError> {
        self.0
            .trim()
            .parse::<super::Value>()
            .map_err(super::ToNumberError::InvalidString)?
            .to_number()
    }

    pub fn to_list<InjectedIntrisic>(
        self,
    ) -> Result<ValueList<InjectedIntrisic>, super::ToListError> {
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

impl From<ValueString> for String {
    fn from(value: ValueString) -> Self {
        value.0.into_string()
    }
}

impl From<char> for ValueString {
    fn from(value: char) -> Self {
        Self(String::from(value).into_boxed_str())
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

#[cfg(feature = "pretty")]
impl<'a, D, A> pretty::Pretty<'a, D, A> for &'a ValueString
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator.text(self.to_string())
    }
}
