//! unary operations

use super::Expression;

/// An unary operator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "bincode", derive(bincode::Decode, bincode::Encode,))]
pub enum UnOp {
    /// `+`: Sum lists and maps, recursive
    Plus,
    /// `-`: Negate. Distribute inside lists and maps
    Neg,
    /// `d`: Throw a dice
    Dice,
}

/// An expression made with an unary operator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode,),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
pub struct ExpressionUnOp<InjectedIntrisic> {
    pub op: UnOp,
    pub expression: Box<Expression<InjectedIntrisic>>,
}

impl<InjectedIntrisic> ExpressionUnOp<InjectedIntrisic> {
    pub fn new(op: UnOp, expression: Expression<InjectedIntrisic>) -> Self {
        Self {
            op,
            expression: Box::new(expression),
        }
    }
}
