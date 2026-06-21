//! Evaluation context

use dices_ast::expr::scope::ScopeInner;
use dices_values::Value;
use dices_values::int::ValueInt;
use num::traits::ConstOne;
use rand::Rng;

use crate::{Engine, EvalError, Evaluator};

/// Evaluation context
pub struct Context<'engine> {
    engine: &'engine mut Engine,
}

impl<'engine> Context<'engine> {
    /// Create a new context
    pub(crate) fn new(engine: &'engine mut Engine) -> Self {
        Self { engine }
    }

    /// Throw a dice
    pub fn dice(&mut self, faces: ValueInt) -> ValueInt {
        let range = if faces > ValueInt::ONE {
            ValueInt::ONE..=faces
        } else {
            faces..=ValueInt::ONE
        };
        self.engine.rng.gen_range(range)
    }

    /// Execute an expression in a scoped context
    ///
    /// The executed expression can read and set variables from the outside
    /// context, but not define new ones.
    pub fn scope<R>(&mut self, fun: impl FnOnce(&mut Context<'_>) -> R) -> R {
        let mut scoped = Context {
            engine: self.engine,
        };
        fun(&mut scoped)
    }
}

impl Evaluator for Context<'_> {
    fn eval(&mut self, stmt: &ScopeInner) -> Result<Value, EvalError> {
        crate::eval::expr::scope::eval_inner(stmt, self)
    }
}
