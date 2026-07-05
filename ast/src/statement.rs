//! Statements
//!
//! Can create variables

use derive_more::{From, IsVariant, TryInto, TryUnwrap, Unwrap};

use crate::{expr::Expr, statement::assign::AssignStatement};

pub mod assign;

/// A statement
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, IsVariant, TryUnwrap, From, TryInto, Unwrap,
)]
pub enum Statement {
    /// Assign statement
    Assign(AssignStatement),
    /// Expression statement
    ///
    /// Evaluated, then value is discarded
    Expr(Expr),
    /// Empty statement
    ///
    /// Returns `null`
    Empty,
}
