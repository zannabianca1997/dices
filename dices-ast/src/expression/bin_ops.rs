//! binary operations

use super::Expression;

/// An unary operator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinOp {
    /// `+`: Sum lists and maps, recursive
    Plus,
    /// `-`: Negate. Distribute inside lists and maps
    Neg,
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
}

/// An expression made with an unary operator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionBinOp {
    pub op: BinOp,
    pub expressions: [Box<Expression>; 2],
}
