use derive_more::derive::{From, Into};
use nunny::NonEmpty;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct ExpressionScope<InjectedIntrisic>(pub Box<NonEmpty<[Expression<InjectedIntrisic>]>>);

impl<InjectedIntrisic> ExpressionScope<InjectedIntrisic> {
    pub fn new(exprs: Box<NonEmpty<[Expression<InjectedIntrisic>]>>) -> Self {
        Self(exprs)
    }
}
