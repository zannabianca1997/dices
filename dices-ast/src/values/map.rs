use std::{collections::BTreeMap, fmt::Display};

use itertools::Itertools;

use crate::fmt::quoted_if_not_ident;

use super::{list::ValueList, string::ValueString, ToNumberError, Value};

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
pub struct ValueMap(BTreeMap<ValueString, Value>);
impl ValueMap {
    pub fn to_number(self) -> Result<super::number::ValueNumber, super::ToNumberError> {
        match self.0.into_iter().exactly_one() {
            Ok((_, value)) => value.to_number(),
            Err(vals) => Err(ToNumberError::WrongListLength(vals.len())),
        }
    }
    pub fn to_list(self) -> Result<ValueList, super::ToListError> {
        Ok(self.0.into_values().collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ValueString, &Value)> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&ValueString, &mut Value)> {
        self.0.iter_mut()
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.0.get_mut(key)
    }
}

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
impl IntoIterator for ValueMap {
    type Item = (ValueString, Value);

    type IntoIter = <BTreeMap<ValueString, Value> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
