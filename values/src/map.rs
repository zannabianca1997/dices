use std::{collections::{BTreeMap}, fmt::Display, ops::Deref, sync::Arc};

use crate::{Value, string::ValueString};

/// Map of strings to values
///
/// Cheap to clone
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ValueMap(Arc<BTreeMap<ValueString, Value>>);

impl ValueMap {
    pub fn new(map: BTreeMap<ValueString, Value>) -> Self {
        Self(Arc::new(map))
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
    type Item=(ValueString,Value);
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        Arc::unwrap_or_clone(self.0).into_iter()
    }
}
