use crate::expr::Expr;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallExpr {
    pub called: Expr,
    pub args: Vec<Expr>,
}
