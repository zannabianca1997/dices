use crate::value::string::ValueString;

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
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr + 'static")
)]
pub struct ExpressionMap<InjectedIntrisic>(Box<[(ValueString, Expression<InjectedIntrisic>)]>);
impl<InjectedIntrisic> ExpressionMap<InjectedIntrisic> {
    pub fn iter(&self) -> impl Iterator<Item = (&ValueString, &Expression<InjectedIntrisic>)> {
        self.0.iter().map(|(a, b)| (a, b))
    }
}

impl<InjectedIntrisic> FromIterator<(ValueString, Expression<InjectedIntrisic>)>
    for ExpressionMap<InjectedIntrisic>
{
    fn from_iter<T: IntoIterator<Item = (ValueString, Expression<InjectedIntrisic>)>>(
        iter: T,
    ) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
