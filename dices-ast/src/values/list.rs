use std::fmt::Display;

use itertools::Itertools;

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
pub struct ValueList(Box<[super::Value]>);

impl Display for ValueList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter().format(", "))
    }
}

impl FromIterator<super::Value> for ValueList {
    fn from_iter<T: IntoIterator<Item = super::Value>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
