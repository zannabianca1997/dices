use std::{collections::BTreeMap, fmt::Display};

use itertools::Itertools;

use crate::fmt::quoted_if_not_ident;

use super::{string::ValueString, Value};

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
pub struct ValueMap(BTreeMap<ValueString, Value>);

impl Display for ValueMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct KeyValue<'m>((&'m ValueString, &'m super::Value));
        impl Display for KeyValue<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let (idx, val) = self.0;
                quoted_if_not_ident(&idx, f)?;
                write!(f, ": {val}")
            }
        }

        write!(f, "<|{}|>", self.0.iter().map(KeyValue).format(", "))
    }
}

impl FromIterator<(ValueString, Value)> for ValueMap {
    fn from_iter<T: IntoIterator<Item = (ValueString, Value)>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
