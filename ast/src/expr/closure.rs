use crate::{expr::Expr, identifier::Identifier};

/// Closure definition
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ClosureExpr {
    pub args: Vec<Identifier>,
    pub body: Expr,
}
