use crate::expr::Expr;

/// A list expression
///
/// This is not a literal as the members can be
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ListExpr {
    pub items: Vec<Expr>,
}
