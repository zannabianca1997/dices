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
pub struct ExpressionList(Box<[Expression]>);

impl FromIterator<Expression> for ExpressionList {
    fn from_iter<T: IntoIterator<Item = Expression>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
