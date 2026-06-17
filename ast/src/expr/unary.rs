use crate::expr::Expr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UnOp {
    // Math
    /// `+` operator
    Plus,
    /// `-` operator
    Minus,
    // Logic
    /// `!` operator
    Not,
    // Misc
    /// `d` operator
    Dice,
}

/// A binary expression
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnaryExpr {
    op: UnOp,
    operand: Expr,
}
