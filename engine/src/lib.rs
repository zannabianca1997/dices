#![doc = include_str!("../README.md")]

use dices_ast::statement::Statement;
use dices_values::{Value, cast::CastInjectedError};
use snafu::Snafu;

use crate::context::Context;

pub mod context;
mod eval;
mod utils;

/// An engine evaluating dices statements
#[derive(Debug, Clone)]
pub struct Engine {}

impl Engine {
    /// Create a new engine
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate a statement
    pub fn eval(&mut self, stmt: &Statement) -> Result<Value, EvalError> {
        eval::statement::eval(stmt, &mut Context::new(self))
    }
}

#[derive(Debug, Snafu)]
pub enum EvalError {
    #[snafu(transparent)]
    CastInjected { source: CastInjectedError },
}
