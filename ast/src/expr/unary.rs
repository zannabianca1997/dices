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

/// A unary expression
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UnaryExpr {
    pub op: UnOp,
    pub operand: Expr,
}
