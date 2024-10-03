use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

use derive_more::derive::{From, Into};
use itertools::Itertools;

use crate::intrisics::{InjectedIntr, NoInjectedIntrisics};

use super::Value;

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
    // Conversions
    From,
    Into,
)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode),
    bincode(bounds = "InjectedIntrisic: InjectedIntr")
)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound = "InjectedIntrisic: InjectedIntr")
)]
pub struct ValueList<InjectedIntrisic>(Box<[Value<InjectedIntrisic>]>);
impl<InjectedIntrisic> ValueList<InjectedIntrisic> {
    #[cfg(feature = "parse_value")]
    pub fn to_number(self) -> Result<super::ValueNumber, super::ToNumberError> {
        match Box::<[_; 1]>::try_from(self.0) {
            Ok(box [value]) => value.to_number(),
            Err(vals) => Err(super::ToNumberError::WrongListLength(vals.len())),
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
impl ValueList<NoInjectedIntrisics> {
    pub fn with_arbitrary_injected_intrisics<II>(self) -> ValueList<II> {
        ValueList(
            self.into_iter()
                .map(Value::with_arbitrary_injected_intrisics)
                .collect(),
        )
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

impl<II> From<Vec<Value<II>>> for ValueList<II> {
    fn from(value: Vec<Value<II>>) -> Self {
        value.into_boxed_slice().into()
    }
}

#[cfg(feature = "pretty")]
impl<'a, D, A, II> pretty::Pretty<'a, D, A> for &'a ValueList<II>
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
    II: InjectedIntr,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator
            .intersperse(self.iter(), crate::fmt::CommaLine)
            .enclose(allocator.line_(), allocator.line_())
            .group()
            .nest(4)
            .enclose("[", "]")
    }
}
