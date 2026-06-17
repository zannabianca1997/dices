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
        // Logic
        /// `==` operator
        Eq,
        /// `!=` operator
        Ne,
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
        lhs: Expr,
        op: BinOp,
        rhs: Expr,
    }
