use derive_more::derive::Display;

use super::{number::ValueNumber, ToNumberError};

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
}
