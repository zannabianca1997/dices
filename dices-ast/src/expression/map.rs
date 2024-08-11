use std::collections::BTreeMap;

use crate::values::string::ValueString;

use super::Expression;

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
pub struct ExpressionMap(BTreeMap<ValueString, Expression>);

impl FromIterator<(ValueString, Expression)> for ExpressionMap {
    fn from_iter<T: IntoIterator<Item = (ValueString, Expression)>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
