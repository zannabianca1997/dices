use std::fmt::Display;

use itertools::Itertools;

use super::{ToNumberError, Value};

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
pub struct ValueList(Box<[Value]>);
impl ValueList {
    pub fn to_number(self) -> Result<super::number::ValueNumber, super::ToNumberError> {
        match Box::<[Value; 1]>::try_from(self.0) {
            Ok(box [value]) => value.to_number(),
            Err(vals) => Err(ToNumberError::WrongListLength(vals.len())),
        }
    }

    fn len(&self) -> usize {
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
