//! binary operations

use super::Expression;

/// An unary operator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinOp {
    /// `+`: Sum lists and maps, recursive
    Add,
    /// `-`: Negate. Distribute inside lists and maps
    Sub,
    /// `~`: Join list and maps
    Join,
    /// `^`: Repeat an operation, building a list
    Repeat,
    /// `*`: multiply, distributing scalars over lists or maps
    Mult,
    /// `%`: remainder, distributing scalars over lists or maps
    Rem,
    /// `/`: divide, distributing scalars over lists or maps
    Div,
    /// `kh`: keep the highest n values of a list or map
    KeepHigh,
    /// `kl`: keep the lowest n values of a list or map
    KeepLow,
    /// `rh`: keep the highest n values of a list or map
    RemoveHigh,
    /// `rl`: keep the lowest n values of a list or map
    RemoveLow,
}

/// An expression made with an unary operator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionBinOp {
    pub op: BinOp,
    pub expressions: Box<[Expression; 2]>,
}

impl ExpressionBinOp {
    pub fn new(op: BinOp, a: Expression, b: Expression) -> Self {
        Self {
            op,
            expressions: Box::new([a, b]),
        }
    }
}
