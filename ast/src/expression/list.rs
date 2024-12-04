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
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode,),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
pub struct ExpressionList<InjectedIntrisic>(Box<[Expression<InjectedIntrisic>]>);
impl<InjectedIntrisic> ExpressionList<InjectedIntrisic> {
    pub fn iter(&self) -> impl Iterator<Item = &Expression<InjectedIntrisic>> {
        self.0.iter()
    }
}

impl<InjectedIntrisic> FromIterator<Expression<InjectedIntrisic>>
    for ExpressionList<InjectedIntrisic>
{
    fn from_iter<T: IntoIterator<Item = Expression<InjectedIntrisic>>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
