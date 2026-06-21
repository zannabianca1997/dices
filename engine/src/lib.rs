#![doc = include_str!("../README.md")]

use dices_ast::{expr::scope::ScopeInner, identifier::Identifier};
use dices_values::{
    Value,
    cast::{CastInjectedError, CastIntoIntError},
    injected::CallError,
};
use rand::SeedableRng;
use snafu::Snafu;

use crate::context::EngineContext;

pub mod context;
mod eval;
mod utils;
mod var_use;

pub trait Evaluator {
    fn eval(&mut self, stmt: &ScopeInner) -> Result<Value, EvalError>;
}

/// An engine evaluating dices statements
#[derive(Debug, Clone)]
pub struct Engine {
    /// Random number generator
    rng: rand_pcg::Lcg64Xsh32,
    /// Global variables
    globals: context::Scope,
}

impl Engine {
    /// Create a new engine
    pub fn new(seed: [u8; 16]) -> Self {
        Self {
            rng: SeedableRng::from_seed(seed),
            globals: context::Scope::new(),
        }
    }
}

impl Evaluator for Engine {
    fn eval(&mut self, stmt: &ScopeInner) -> Result<Value, EvalError> {
        eval::expr::scope::eval_inner(stmt, &mut EngineContext::new(self))
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
    #[snafu(display("Unknown variable {name}"))]
    VariableDoNotExists { name: Identifier },
    #[snafu(display("Error in calling value of type {}", value.typ()))]
    Call { value: Value, source: CallError },
}
