use crate::{
    expr::{binary::BinaryExpr, unary::UnaryExpr},
    literal::Literal,
};

pub mod binary;
pub mod unary;

/// An expression
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expr {
    Literal(Box<Literal>),
    Binary(Box<BinaryExpr>),
    Unary(Box<UnaryExpr>),
}
