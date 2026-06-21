use std::sync::Arc;

use derive_more::{From, IsVariant, TryInto, TryUnwrap, Unwrap};
use dices_values::Value;

use crate::{identifier::Identifier, literal::Literal};
use {
    binary::BinaryExpr, call::CallExpr, closure::ClosureExpr, list::ListExpr, map::MapExpr,
    scope::ScopeExpr, unary::UnaryExpr,
};

pub mod binary;
pub mod call;
pub mod closure;
pub mod list;
pub mod map;
pub mod scope;
pub mod unary;

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
    /// Reference to variable
    Variable(Box<Identifier>),
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
    /// Closure expression
    // In an [`Arc`] as the generated elements will borrow the expression
    Closure(Arc<ClosureExpr>),
    /// Call expression
    Call(Box<CallExpr>),
}
