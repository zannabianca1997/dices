//! The `null` value

use std::fmt::Display;

use super::{ToNumberError, ValueList};

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
)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct ValueNull;
impl ValueNull {
    pub fn to_number(self) -> Result<super::ValueNumber, super::ToNumberError> {
        Err(ToNumberError::InvalidNull)
    }
    pub fn to_list(self) -> Result<super::ValueList, super::ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}

impl Display for ValueNull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}
