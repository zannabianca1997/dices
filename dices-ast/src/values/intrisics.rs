use std::fmt::Display;

use derive_more::derive::From;

use crate::intrisics::Intrisic;

use super::{ToListError, ToNumberError, ValueList, ValueNumber};

/// Value representing an intrisic
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
    // conversion
    From,
)]
pub struct ValueIntrisic(Intrisic);

impl Display for ValueIntrisic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<intrisic {}>", <&'static str>::from(self.0))
    }
}

impl ValueIntrisic {
    pub fn to_number(&self) -> Result<ValueNumber, ToNumberError> {
        Err(ToNumberError::Intrisic)
    }
    pub fn to_list(self) -> Result<ValueList, ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}
