use std::{
    fmt::{Debug, Display, Write},
    hash::Hash,
    iter::{FusedIterator, chain},
    ops::{Deref},
    slice::SliceIndex,
    sync::Arc,
    vec,
};

use yoke::Yoke;

use crate::Value;

/// A list of values
///
/// Cheaply cloneable and sliceable
#[derive(Clone)]
pub struct ValueList(Yoke<&'static [Value], Option<Arc<Vec<Value>>>>);

impl ValueList {
    /// Create a new list
    pub fn new(values: Vec<Value>) -> Self {
        Self(Yoke::attach_to_cart(Arc::new(values), |cart| cart.as_slice()).wrap_cart_in_option())
    }

    /// The empty list
    pub const fn empty() -> Self {
        Self(Yoke::new_owned(&[]))
    }

    /// Slice of values
    pub fn as_slice(&self) -> &[Value] {
        self.0.get()
    }

    /// Get a sublist
    ///
    /// This will obtain a sublist that references the same backing list
    pub fn slice<I>(&self, i: I) -> Option<Self>
    where
        I: SliceIndex<[Value], Output = [Value]>,
    {
        let inner = self
            .0
            .try_map_project_cloned(|s, _| s.get(i).ok_or(()))
            .ok()?;
        Some(Self(inner))
    }

    /// Concatenate two lists
    pub fn concat(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }

        Self::from_iter(chain(self, other))
    }
}

impl AsRef<[Value]> for ValueList {
    fn as_ref(&self) -> &[Value] {
        self.as_slice()
    }
}

impl Deref for ValueList {
    type Target = [Value];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

pub struct IntoIter(IntoIterInner);
enum IntoIterInner {
    Cloning(Yoke<&'static [Value], Option<Arc<Vec<Value>>>>),
    FromVec(vec::IntoIter<Value>),
}

impl IntoIterator for ValueList {
    type Item = Value;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        if self
            .0
            .backing_cart()
            .as_deref()
            .is_some_and(|cart| cart.len() == self.len())
        {
            match Arc::try_unwrap(self.0.into_backing_cart().unwrap()) {
                Ok(v) => IntoIter(IntoIterInner::FromVec(v.into_iter())),
                Err(arc) => IntoIter(IntoIterInner::Cloning(
                    Yoke::attach_to_cart(arc, |c| c.as_slice()).wrap_cart_in_option(),
                )),
            }
        } else {
            IntoIter(IntoIterInner::Cloning(self.0))
        }
    }
}

impl Iterator for IntoIter {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            IntoIterInner::Cloning(yoke) => {
                let res = yoke.with_mut_return(|s| s.split_off_first().cloned());
                if res.is_none() {
                    *self = Self(IntoIterInner::FromVec(vec![].into_iter()))
                }
                res
            }
            IntoIterInner::FromVec(vec_iter) => vec_iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl ExactSizeIterator for IntoIter {
    fn len(&self) -> usize {
        match &self.0 {
            IntoIterInner::Cloning(yoke) => yoke.get().len(),
            IntoIterInner::FromVec(iter) => iter.len(),
        }
    }
}

impl FusedIterator for IntoIter {}

impl DoubleEndedIterator for IntoIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            IntoIterInner::Cloning(yoke) => {
                let res = yoke.with_mut_return(|s| s.split_off_last().cloned());
                if res.is_none() {
                    *self = Self(IntoIterInner::FromVec(vec![].into_iter()))
                }
                res
            }
            IntoIterInner::FromVec(vec_iter) => vec_iter.next_back(),
        }
    }
}

impl FromIterator<Value> for ValueList {
    fn from_iter<T: IntoIterator<Item = Value>>(iter: T) -> Self {
        Self::new(Vec::from_iter(iter))
    }
}

impl Debug for ValueList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValueList").field(&self.as_slice()).finish()
    }
}

impl PartialEq<ValueList> for [Value] {
    fn eq(&self, other: &ValueList) -> bool {
        self.eq(other.as_slice())
    }
}
impl PartialEq<&ValueList> for [Value] {
    fn eq(&self, other: &&ValueList) -> bool {
        self.eq(other.as_slice())
    }
}
impl<Rhs> PartialEq<Rhs> for ValueList
where
    [Value]: PartialEq<Rhs>,
{
    fn eq(&self, other: &Rhs) -> bool {
        self.as_slice().eq(other)
    }
}
impl Eq for ValueList {}

impl PartialOrd<ValueList> for [Value] {
    fn partial_cmp(&self, other: &ValueList) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_slice())
    }
}
impl PartialOrd<&ValueList> for [Value] {
    fn partial_cmp(&self, other: &&ValueList) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_slice())
    }
}
impl<Rhs> PartialOrd<Rhs> for ValueList
where
    [Value]: PartialOrd<Rhs>,
{
    fn partial_cmp(&self, other: &Rhs) -> Option<std::cmp::Ordering> {
        self.as_slice().partial_cmp(other)
    }
}
impl Ord for ValueList {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl Hash for ValueList {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl Display for ValueList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_char('[')?;
        if let Some((first, follows)) = self.split_first() {
            write!(f, "{first}")?;
            for el in follows {
                write!(f, ", {el}")?;
            }
        }
        f.write_char(']')
    }
}

impl Default for ValueList {
    fn default() -> Self {
        Self::empty()
    }
}
