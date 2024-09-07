use std::ops::{Deref, DerefMut};

use derive_more::derive::{From, Into};
use nunny::NonEmpty;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct ExpressionScope<InjectedIntrisic>(Box<NonEmpty<[Expression<InjectedIntrisic>]>>);

impl<InjectedIntrisic> Deref for ExpressionScope<InjectedIntrisic> {
    type Target = NonEmpty<[Expression<InjectedIntrisic>]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<InjectedIntrisic> DerefMut for ExpressionScope<InjectedIntrisic> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<InjectedIntrisic> ExpressionScope<InjectedIntrisic> {
    pub fn new(exprs: Box<NonEmpty<[Expression<InjectedIntrisic>]>>) -> Self {
        Self(exprs)
    }
}
