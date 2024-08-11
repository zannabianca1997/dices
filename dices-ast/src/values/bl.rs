use derive_more::derive::Display;

use super::{list::ValueList, number::ValueNumber, ToNumberError};

/// A signed integer value
#[derive(
    // display helper
    Debug,
    Display,
    // cloning
    Clone,
    Copy,
    // comparisons
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
pub struct ValueBool(bool);
impl ValueBool {
    pub fn to_number(self) -> Result<ValueNumber, ToNumberError> {
        Ok(match self.0 {
            true => ValueNumber::ONE,
            false => ValueNumber::ZERO,
        })
    }

    pub fn to_list(self) -> Result<super::list::ValueList, super::ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}
