use std::fmt::Display;

use derive_more::derive::{From, Into};

use crate::intrisics::{InjectedIntr, Intrisic};

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
    Into,
)]
pub struct ValueIntrisic<Injected>(Intrisic<Injected>);

impl<Injected: InjectedIntr> Display for ValueIntrisic<Injected> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<intrisic {}>", self.0.name())
    }
}

impl<Injected> ValueIntrisic<Injected> {
    pub fn to_number(&self) -> Result<ValueNumber, ToNumberError> {
        Err(ToNumberError::Intrisic)
    }
    pub fn to_list(self) -> Result<ValueList<Injected>, ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}
