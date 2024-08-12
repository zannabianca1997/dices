use std::fmt::Display;

use itertools::Itertools;

use super::{number::ValueNumber, ToNumberError, Value};

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
pub struct ValueList(Box<[Value]>);
impl ValueList {
    pub fn to_number(self) -> Result<ValueNumber, super::ToNumberError> {
        match Box::<[Value; 1]>::try_from(self.0) {
            Ok(box [value]) => value.to_number(),
            Err(vals) => Err(ToNumberError::WrongListLength(vals.len())),
        }
    }

    pub fn to_list(self) -> Result<ValueList, super::ToListError> {
        Ok(self)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Display for ValueList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter().format(", "))
    }
}

impl FromIterator<Value> for ValueList {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}