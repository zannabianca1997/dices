//! Context essential to evaluate a `dices` expression

use std::{collections::BTreeMap, mem};

use dices_ast::{ident::IdentStr, values::Value};
use nunny::NonEmpty;

type Scope = BTreeMap<Box<IdentStr>, Value>;

#[derive(Debug, Clone)]
pub struct Context<RNG> {
    /// the stack of variables
    scopes: NonEmpty<Vec<Scope>>,
    /// The random number generator
    rng: RNG,
}

impl<RNG> Context<RNG> {
    pub fn new(rng: RNG) -> Self {
        Self {
            scopes: nunny::vec![Scope::new()],
            rng,
        }
    }

    /// run code in a local scope, with the same RNG and no local variables
    pub fn scoped<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        self.scopes.push(Scope::new());
        let res = f(self);
        unsafe {
            // SAFETY: pushing and popping is balanced.
            // We just pushed on a non empty vector, so we can
            // pop without emptying it.
            self.scopes.as_mut_vec().pop()
        };
        res
    }

    /// run code in a jail, with the same RNG but no variables
    pub fn jailed<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> R {
        let old_scopes = mem::replace(&mut self.scopes, nunny::vec![Scope::new()]);
        let res = f(self);
        self.scopes = old_scopes;
        res
    }

    /// Obtain a readonly handle to the variables
    pub fn vars(&self) -> Vars {
        Vars(&self.scopes)
    }
    /// Obtain an handle to the variables
    pub fn vars_mut(&mut self) -> VarsMut {
        VarsMut(&mut self.scopes)
    }

    /// Obtain an handle to the rng
    pub fn rng(&mut self) -> &mut RNG {
        &mut self.rng
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vars<'c>(&'c NonEmpty<[Scope]>);

impl Vars<'_> {
    /// Find the value of a variable
    pub fn get(&self, name: &IdentStr) -> Option<&Value> {
        // find the last scope that contains that variable
        self.0.iter().rev().find_map(|s| s.get(name))
    }
}
impl<'c> From<VarsMut<'c>> for Vars<'c> {
    fn from(value: VarsMut<'c>) -> Self {
        Self(&*value.0)
    }
}
impl<'c> From<&'c VarsMut<'c>> for Vars<'c> {
    fn from(value: &'c VarsMut<'c>) -> Self {
        Self(&*value.0)
    }
}
impl<'c> From<&'c mut VarsMut<'c>> for Vars<'c> {
    fn from(value: &'c mut VarsMut<'c>) -> Self {
        Self(&*value.0)
    }
}

#[derive(Debug)]
pub struct VarsMut<'c>(&'c mut NonEmpty<[Scope]>);

impl VarsMut<'_> {
    /// Let a variable be, setting its value if present in the current scope, or creating it
    pub fn let_(&mut self, name: Box<IdentStr>, value: Value) {
        self.0.last_mut().insert(name, value);
    }
    /// Find the value of a variable
    pub fn get(&self, name: &IdentStr) -> Option<&Value> {
        // find the last scope that contains that variable
        self.0.iter().rev().find_map(|s| s.get(name))
    }
    /// Find the value of a variable, and permit to modify it
    pub fn get_mut(&mut self, name: &IdentStr) -> Option<&mut Value> {
        // find the last scope that contains that variable
        self.0.iter_mut().rev().find_map(|s| s.get_mut(name))
    }
}
