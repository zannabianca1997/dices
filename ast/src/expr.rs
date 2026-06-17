use derive_more::{From, IsVariant, TryInto, TryUnwrap, Unwrap};

use crate::{
    expr::{binary::BinaryExpr, unary::UnaryExpr},
    literal::Literal,
};

pub mod binary;
pub mod unary;

/// An expression
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IsVariant, TryUnwrap, From, TryInto, Unwrap,
)]
pub enum Expr {
    Literal(Box<Literal>),
    Binary(Box<BinaryExpr>),
    Unary(Box<UnaryExpr>),
}
