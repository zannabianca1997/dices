use std::{collections::BTreeMap, fmt::Display, ops::Deref, sync::Arc};

use crate::{Value, string::ValueString};

/// Map of strings to values
///
/// Cheap to clone
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ValueMap(Arc<BTreeMap<ValueString, Value>>);

impl ValueMap {
    pub fn new(map: BTreeMap<ValueString, Value>) -> Self {
        Self(Arc::new(map))
    }

    /// Join two maps, with the keys from `other` taking precedence
    pub fn join(mut self, other: Self) -> Self {
        // `self` is fully covered by `other` (or empty)
        if self.keys().all(|k| other.contains_key(k)) {
            return other;
        }
        // `other` is empty
        if other.is_empty() {
            return self;
        }

        let map = Arc::make_mut(&mut self.0);
        for (k, v) in other.0.iter() {
            map.insert(k.clone(), v.clone());
        }
        self
    }
}

impl Deref for ValueMap {
    type Target = BTreeMap<ValueString, Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for ValueMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<|")?;
        let mut iter = self.iter();
        if let Some((key, value)) = iter.next() {
            write!(f, "{key}: {value}")?;
            for (key, value) in iter {
                write!(f, ", {key}: {value}")?;
            }
        }
        f.write_str("|>")
    }
}

pub type IntoIter = std::collections::btree_map::IntoIter<ValueString, Value>;

impl IntoIterator for ValueMap {
    type Item = (ValueString, Value);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Arc::unwrap_or_clone(self.0).into_iter()
    }
}

impl FromIterator<(ValueString, Value)> for ValueMap {
    fn from_iter<T: IntoIterator<Item = (ValueString, Value)>>(iter: T) -> Self {
        Self::new(FromIterator::from_iter(iter))
    }
}
