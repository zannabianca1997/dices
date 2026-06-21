use derive_more::{From, IsVariant, TryInto, TryUnwrap, Unwrap};
use dices_values::Value;

use crate::{
    expr::{binary::BinaryExpr, list::ListExpr, map::MapExpr, scope::ScopeExpr, unary::UnaryExpr},
    literal::Literal,
};

pub mod binary;
pub mod scope;
pub mod unary;
pub mod list {
    use crate::expr::Expr;

    /// A list expression
    ///
    /// This is not a literal as the members can be
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ListExpr {
        pub items: Vec<Expr>,
    }
}
pub mod map {
    use crate::{expr::Expr, literal::LiteralString};

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct MapExpr {
        pub items: Vec<(LiteralString, Expr)>,
    }
}

/// An expression
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IsVariant, TryUnwrap, From, TryInto, Unwrap,
)]
pub enum Expr {
    /// Constant expression
    ///
    /// This is never produced by the parser, is instead generated during const
    /// evaluation
    Const(Box<Value>),
    /// Literal expression
    Literal(Box<Literal>),
    /// List expression
    List(Box<ListExpr>),
    /// Map expression
    Map(Box<MapExpr>),
    /// Binary operation
    Binary(Box<BinaryExpr>),
    /// Unary operation
    Unary(Box<UnaryExpr>),
    /// Scope expression
    Scope(Box<ScopeExpr>),
}
