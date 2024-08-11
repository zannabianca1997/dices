//! Value enclosing an expression

use super::Expression;
use crate::ident::IdentStr;

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
pub struct ExpressionClosure {
    pub params: Box<[Box<IdentStr>]>,
    pub body: Box<Expression>,
}
