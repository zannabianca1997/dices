//! Evaluation context

use std::mem;
use std::{collections::BTreeMap, iter::once};

use dices_ast::identifier::Identifier;
use dices_values::Value;
use dices_values::injected::call::InjectedContext;
use dices_values::int::ValueInt;
use dices_values::string::ValueString;
use num::traits::ConstOne;
use rand::Rng;

use crate::Engine;

pub(crate) trait Context {
    /// Throw a dice
    fn dice(&mut self, faces: ValueInt) -> ValueInt;

    type Scoped: Context + ?Sized;
    /// Execute an expression in a scoped context
    ///
    /// The executed expression can read and set variables from the outside
    /// context, but not define new ones.
    fn scope<R>(&mut self, fun: impl FnOnce(&mut Self::Scoped) -> R) -> R;

    type Jailed: Context + ?Sized;
    /// Execute an expression in a jailed context
    ///
    /// The executed expression won't be able to modify or read any variable
    /// from the external scope.
    fn jail<R>(&mut self, fun: impl FnOnce(&mut Self::Jailed) -> R) -> R;

    /// Create a variable
    ///
    /// If it exists in the current scope, shadows it
    fn let_var(&mut self, name: Identifier, value: Value);

    /// Get a variable value
    fn var(&self, name: &Identifier) -> Option<&Value>;

    /// Get a mutable variable value
    fn var_mut(&mut self, name: &Identifier) -> Option<&mut Value>;

    fn inject(&mut self) -> &mut dyn InjectedContext;
}

/// Evaluation context
pub struct EngineContext<'engine> {
    engine: &'engine mut Engine,
    scopes: Vec<Scope>,
}

impl<'engine> EngineContext<'engine> {
    /// Create a new context
    pub(crate) fn new(engine: &'engine mut Engine) -> Self {
        Self {
            engine,
            scopes: vec![],
        }
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
}
impl<'engine> Context for EngineContext<'engine> {
    fn dice(&mut self, faces: ValueInt) -> ValueInt {
        let range = if faces > ValueInt::ONE {
            ValueInt::ONE..=faces
        } else {
            faces..=ValueInt::ONE
        };
        self.engine.rng.gen_range(range)
    }

    fn let_var(&mut self, name: Identifier, value: Value) {
        self.scopes_mut().next().unwrap().vars.insert(name, value);
    }

    fn var(&self, name: &Identifier) -> Option<&Value> {
        self.scopes().find_map(|s| s.vars.get(name))
    }

    fn var_mut(&mut self, name: &Identifier) -> Option<&mut Value> {
        self.scopes_mut().find_map(|s| s.vars.get_mut(name))
    }

    type Scoped = Self;

    fn scope<R>(&mut self, fun: impl FnOnce(&mut Self::Scoped) -> R) -> R {
        self.scopes.push(Scope::new());
        let res = fun(self);
        self.scopes.pop().unwrap();
        res
    }

    type Jailed = Self;

    fn jail<R>(&mut self, fun: impl FnOnce(&mut Self::Jailed) -> R) -> R {
        let globals = mem::take(&mut self.engine.globals);
        let scopes = mem::take(&mut self.scopes);

        let res = fun(self);

        self.engine.globals = globals;
        self.scopes = scopes;

        res
    }

    fn inject(&mut self) -> &mut dyn InjectedContext {
        self
    }
}

impl InjectedContext for EngineContext<'_> {
    fn dice(&mut self, faces: ValueInt) -> ValueInt {
        Context::dice(self, faces)
    }

    fn enter_scope(&mut self) -> Box<dyn std::any::Any> {
        self.scopes.push(Scope::new());
        Box::new(())
    }

    fn exit_scope(&mut self, _: Box<dyn std::any::Any>) {
        self.scopes.pop().unwrap();
    }

    fn enter_jail(&mut self) -> Box<dyn std::any::Any> {
        let globals = mem::take(&mut self.engine.globals);
        let scopes = mem::take(&mut self.scopes);

        Box::new((globals, scopes))
    }

    fn exit_jail(&mut self, data: Box<dyn std::any::Any>) {
        let (globals, scopes) = *data.downcast().unwrap();

        self.engine.globals = globals;
        self.scopes = scopes;
    }

    fn let_var(&mut self, name: ValueString, value: Value) {
        Context::let_var(self, Identifier::new(name).unwrap(), value);
    }

    fn var(&self, name: &ValueString) -> Option<&Value> {
        Context::var(self, Identifier::new_ref(name).unwrap())
    }

    fn var_mut(&mut self, name: &ValueString) -> Option<&mut Value> {
        Context::var_mut(self, Identifier::new_ref(name).unwrap())
    }
}

impl<'a> Context for dyn InjectedContext + 'a {
    fn dice(&mut self, faces: ValueInt) -> ValueInt {
        InjectedContext::dice(self, faces)
    }

    type Scoped = Self;

    fn scope<R>(&mut self, fun: impl FnOnce(&mut Self::Scoped) -> R) -> R {
        let data = self.enter_scope();
        let res = fun(self);
        self.exit_scope(data);
        res
    }

    type Jailed = Self;

    fn jail<R>(&mut self, fun: impl FnOnce(&mut Self::Jailed) -> R) -> R {
        let data = self.enter_jail();
        let res = fun(self);
        self.exit_jail(data);
        res
    }

    fn let_var(&mut self, name: Identifier, value: Value) {
        InjectedContext::let_var(self, name.into(), value);
    }

    fn var(&self, name: &Identifier) -> Option<&Value> {
        InjectedContext::var(self, name.as_ref())
    }

    fn var_mut(&mut self, name: &Identifier) -> Option<&mut Value> {
        InjectedContext::var_mut(self, name.as_ref())
    }

    fn inject(&mut self) -> &mut dyn InjectedContext {
        self
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
