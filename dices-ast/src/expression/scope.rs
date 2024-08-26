use derive_more::derive::{From, Into};
use nunny::NonEmpty;

use super::Expression;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From, Into)]
pub struct ExpressionScope(pub Box<NonEmpty<[Expression]>>);

impl ExpressionScope {
    pub fn new(exprs: Box<NonEmpty<[Expression]>>) -> Self {
        Self(exprs)
    }
}
