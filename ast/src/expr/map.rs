use crate::{expr::Expr, literal::LiteralString};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MapExpr {
    pub items: Vec<(LiteralString, Expr)>,
}
