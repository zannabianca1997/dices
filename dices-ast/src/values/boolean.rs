use derive_more::derive::{Display, From, Into};

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
    // conversions
    From,
    Into,
)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct ValueBool(bool);
impl ValueBool {
    pub const TRUE: Self = ValueBool(true);
    pub const FALSE: Self = ValueBool(false);

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
