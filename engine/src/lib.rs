#![doc = include_str!("../README.md")]

use dices_ast::expr::scope::ScopeInner;
use dices_values::{
    Value,
    cast::{CastInjectedError, CastIntoIntError},
};
use rand::SeedableRng;
use snafu::Snafu;

use crate::context::Context;

pub mod context;
mod eval;
mod utils;

pub trait Evaluator {
    fn eval(&mut self, stmt: &ScopeInner) -> Result<Value, EvalError>;
}

/// An engine evaluating dices statements
#[derive(Debug, Clone)]
pub struct Engine {
    rng: rand_pcg::Lcg64Xsh32,
}

impl Engine {
    /// Create a new engine
    pub fn new(seed: [u8; 16]) -> Self {
        Self {
            rng: SeedableRng::from_seed(seed),
        }
    }
}

impl Evaluator for Engine {
    fn eval(&mut self, stmt: &ScopeInner) -> Result<Value, EvalError> {
        eval::expr::scope::eval_inner(stmt, &mut Context::new(self))
    }
}

#[derive(Debug, Snafu)]
pub enum EvalError {
    #[snafu(transparent)]
    CastInjected { source: CastInjectedError },
    #[snafu(transparent)]
    CastIntoInt { source: CastIntoIntError },
    #[snafu(display("Division by zero"))]
    DivisionByZero,
    #[snafu(display("Multiplication between two non scalar"))]
    MulBetweenNonScalars {
        lhs: CastIntoIntError,
        rhs: CastIntoIntError,
    },
}
