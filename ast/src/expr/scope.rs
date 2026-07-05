//! Scope expression
//!
//! ```text
//! "{" { statement ";" }* { instruction }? "}"
//! ```

use crate::{expr::Expr, statement::Statement};

/// Inside of a scope expression
///
/// Also what the REPL command is parsed as
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeInner {
    pub statements: Vec<Statement>,
    pub expr: Option<Expr>,
}

/// Scope expression
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopeExpr(pub ScopeInner);

impl From<Statement> for ScopeInner {
    fn from(value: Statement) -> Self {
        Self {
            statements: vec![value],
            expr: None,
        }
    }
}

impl From<Expr> for ScopeInner {
    fn from(value: Expr) -> Self {
        Self {
            statements: vec![],
            expr: Some(value),
        }
    }
}
