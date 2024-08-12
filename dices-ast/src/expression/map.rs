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
pub struct ExpressionMap(Box<[(ValueString, Expression)]>);
impl ExpressionMap {
    pub fn iter(&self) -> impl Iterator<Item = (&ValueString, &Expression)> {
        self.0.iter().map(|(a, b)| (a, b))
    }
}

impl FromIterator<(ValueString, Expression)> for ExpressionMap {
    fn from_iter<T: IntoIterator<Item = (ValueString, Expression)>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
