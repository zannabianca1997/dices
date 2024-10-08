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
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode,),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
pub struct ExpressionClosure<InjectedIntrisic> {
    pub params: Box<[Box<IdentStr>]>,
    pub body: Box<Expression<InjectedIntrisic>>,
}

impl<InjectedIntrisic> ExpressionClosure<InjectedIntrisic> {
    pub fn new(params: Box<[Box<IdentStr>]>, body: Expression<InjectedIntrisic>) -> Self {
        Self {
            params,
            body: Box::new(body),
        }
    }
}
