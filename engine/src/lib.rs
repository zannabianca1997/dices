#![doc = include_str!("../README.md")]

use dices_ast::statement::Statement;
use dices_values::Value;

pub mod context {
    //! Evaluation context

    use crate::Engine;

    /// Evaluation context
    pub struct Context<'engine> {
        engine: &'engine mut Engine,
    }
}

/// An engine evaluating dices statements
#[derive(Debug, Clone)]
pub struct Engine {}

impl Engine {
    /// Create a new engine
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate a statement
    pub fn eval(&mut self, stmt: &Statement) -> Value {
        todo!()
    }
}
