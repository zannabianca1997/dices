#![doc = include_str!("../README.md")]

use dices_ast::statement::Statement;
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

type RngImpl = rand_pcg::Lcg64Xsh32;

/// An engine evaluating dices statements
#[derive(Debug, Clone)]
pub struct Engine {
    rng: RngImpl,
}

impl Engine {
    /// Create a new engine
    pub fn new(seed: <RngImpl as SeedableRng>::Seed) -> Self {
        Self {
            rng: RngImpl::from_seed(seed),
        }
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
    #[snafu(transparent)]
    CastIntoInt { source: CastIntoIntError },
    DivisionByZero,
}
