use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use itertools::Itertools;

use crate::intrisics::InjectedIntr;

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
pub struct ValueList<InjectedIntrisic>(Box<[Value<InjectedIntrisic>]>);
impl<InjectedIntrisic> ValueList<InjectedIntrisic> {
    pub fn to_number(self) -> Result<ValueNumber, super::ToNumberError> {
        match Box::<[_; 1]>::try_from(self.0) {
            Ok(box [value]) => value.to_number(),
            Err(vals) => Err(ToNumberError::WrongListLength(vals.len())),
        }
    }

    pub fn to_list(self) -> Result<ValueList<InjectedIntrisic>, super::ToListError> {
        Ok(self)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Value<InjectedIntrisic>> {
        self.0.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Value<InjectedIntrisic>> {
        self.0.iter_mut()
    }
}
impl<InjectedIntrisic> Deref for ValueList<InjectedIntrisic> {
    type Target = [Value<InjectedIntrisic>];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<InjectedIntrisic> DerefMut for ValueList<InjectedIntrisic> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<II: InjectedIntr> Display for ValueList<II> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]", self.0.iter().format(", "))
    }
}

impl<InjectedIntrisic> FromIterator<Value<InjectedIntrisic>> for ValueList<InjectedIntrisic> {
    fn from_iter<T: IntoIterator<Item = Value<InjectedIntrisic>>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
impl<InjectedIntrisic> IntoIterator for ValueList<InjectedIntrisic> {
    type Item = Value<InjectedIntrisic>;

    type IntoIter = <Vec<Value<InjectedIntrisic>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_vec().into_iter()
    }
}
