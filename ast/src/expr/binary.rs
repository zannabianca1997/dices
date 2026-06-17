use crate::expr::Expr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinOp {
    // Math
    /// `+` operator
    Add,
    /// `-` operator
    Sub,
    /// `*` operator
    Mul,
    /// `/` operator
    Div,
    /// `%` operator
    Rem,
    // Comparison
    /// `==` operator
    Eq,
    /// `!=` operator
    Ne,
    /// `<` operator
    Lt,
    /// `>` operator
    Gt,
    /// `<=` operator
    Le,
    /// `>=` operator
    Ge,
    // Logic
    /// `&&` operator
    And,
    /// `||` operator
    Or,
    // Misc
    /// `~` operator
    Join,
    /// `d` operator
    Dice,
    /// `^` operator
    Repeat,
}

/// A binary expression
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BinaryExpr {
    pub lhs: Expr,
    pub op: BinOp,
    pub rhs: Expr,
}
