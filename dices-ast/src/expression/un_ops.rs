//! unary operations

use super::Expression;

/// An unary operator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
pub struct ExpressionUnOp {
    pub op: UnOp,
    pub expression: Box<Expression>,
}

impl ExpressionUnOp {
    pub fn new(op: UnOp, expression: Expression) -> Self {
        Self {
            op,
            expression: Box::new(expression),
        }
    }
}
