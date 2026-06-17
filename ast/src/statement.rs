//! Statements
//!
//! Can create variables

use derive_more::{From, IsVariant, TryInto, TryUnwrap, Unwrap};

use crate::expr::Expr;

/// A statement
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IsVariant, TryUnwrap, From, TryInto, Unwrap,
)]
pub enum Statement {
    Expr(Expr),
}
