use crate::expr::Expr;

/// Member access
///
/// This is generated both on explicit indexing `a["b"]` and with dot notation
/// `a.b`, `a.0`, `a."b"`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemberAccessExpr {
    pub container: Expr,
    pub index: Expr,
}
