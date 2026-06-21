//! Evaluation context

use std::{borrow::Borrow, collections::BTreeMap, iter::once};

use dices_ast::{expr::scope::ScopeInner, identifier::Identifier};
use dices_values::Value;
use dices_values::int::ValueInt;
use num::traits::ConstOne;
use rand::Rng;

use crate::{Engine, EvalError, Evaluator};

/// Evaluation context
pub struct Context<'engine> {
    engine: &'engine mut Engine,
    scopes: Vec<Scope>,
}

impl<'engine> Context<'engine> {
    /// Create a new context
    pub(crate) fn new(engine: &'engine mut Engine) -> Self {
        Self {
            engine,
            scopes: vec![],
        }
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
        self.scopes.push(Scope::new());
        let res = fun(self);
        self.scopes.pop().unwrap();
        res
    }

    fn scopes(&self) -> impl Iterator<Item = &Scope> {
        self.scopes.iter().rev().chain(once(&self.engine.globals))
    }
    fn scopes_mut(&mut self) -> impl Iterator<Item = &mut Scope> {
        self.scopes
            .iter_mut()
            .rev()
            .chain(once(&mut self.engine.globals))
    }

    /// Create a variable
    ///
    /// If it exists in the current scope, shadows it
    pub fn let_var(&mut self, name: Identifier, value: Value) {
        self.scopes_mut().next().unwrap().vars.insert(name, value);
    }
    /// Get a variable value
    pub fn var<Q>(&self, name: &Q) -> Option<&Value>
    where
        Q: ?Sized + Ord,
        Identifier: Borrow<Q>,
    {
        self.scopes().find_map(|s| s.vars.get(name))
    }
    /// Get a mutable variable value
    pub fn var_mut<Q>(&mut self, name: &Q) -> Option<&mut Value>
    where
        Q: ?Sized + Ord,
        Identifier: Borrow<Q>,
    {
        self.scopes_mut().find_map(|s| s.vars.get_mut(name))
    }
}

impl Evaluator for Context<'_> {
    fn eval(&mut self, stmt: &ScopeInner) -> Result<Value, EvalError> {
        crate::eval::expr::scope::eval_inner(stmt, self)
    }
}

#[derive(Debug, Clone)]
pub struct Scope {
    vars: BTreeMap<Identifier, Value>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            vars: BTreeMap::new(),
        }
    }
}

impl Default for Scope {
    fn default() -> Self {
        Self::new()
    }
}
