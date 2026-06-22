//! Evaluation context

use std::hash::{Hash, Hasher};
use std::mem;
use std::{collections::BTreeMap, iter::once};

use dices_ast::identifier::Identifier;
use dices_values::Value;
use dices_values::injected::call::InjectedContext;
use dices_values::int::ValueInt;
use dices_values::serde::de::ValueDeserializer;
use dices_values::serde::ser::ValueSerializer;
use num::FromPrimitive;
use num::traits::ConstOne;
use rand::Rng;
use rand_seeder::Seeder;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::Engine;

pub(crate) trait Context {
    /// Seed the random number generator
    fn rng_seed(&mut self, seed: impl Hash);

    /// Serialize the random number generator state
    fn rng_save<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error>;

    /// Restore the random number generator state
    fn rng_restore<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error>;

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
    fn rng_seed(&mut self, seed: impl Hash) {
        self.engine.rng = Seeder::from(seed).make_rng();
    }

    fn rng_save<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.engine.rng.serialize(serializer)
    }
    fn rng_restore<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        self.engine.rng = Deserialize::deserialize(deserializer)?;
        Ok(())
    }

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
    fn rng_seed(&mut self, seed: &[Value]) {
        Context::rng_seed(self, seed);
    }

    fn rng_save(&self, serializer: ValueSerializer) -> dices_values::serde::error::Result<Value> {
        Context::rng_save(self, serializer)
    }

    fn rng_restore(
        &mut self,
        deserializer: ValueDeserializer,
    ) -> dices_values::serde::error::Result<()> {
        Context::rng_restore(self, deserializer)
    }

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

    fn let_var(&mut self, name: Identifier, value: Value) {
        Context::let_var(self, name, value);
    }

    fn var(&self, name: &Identifier) -> Option<&Value> {
        Context::var(self, name)
    }

    fn var_mut(&mut self, name: &Identifier) -> Option<&mut Value> {
        Context::var_mut(self, name)
    }
}

impl<'a> Context for dyn InjectedContext + 'a {
    fn rng_seed(&mut self, seed: impl Hash) {
        /// Hasher storing all bytes as values
        struct HashToValues(Vec<Value>);
        impl Hasher for HashToValues {
            fn finish(&self) -> u64 {
                let bytes = Seeder::from(&self.0).make_seed();
                u64::from_le_bytes(bytes)
            }

            fn write(&mut self, bytes: &[u8]) {
                let (chunks, rem) = bytes.as_chunks();
                for chunk in chunks {
                    self.0.push(
                        ValueInt::from_i64(i64::from_le_bytes(*chunk))
                            .unwrap()
                            .into(),
                    );
                }
                let mut remaining = [0; _];
                remaining[..rem.len()].copy_from_slice(rem);
                self.0.push(
                    ValueInt::from_i64(i64::from_le_bytes(remaining))
                        .unwrap()
                        .into(),
                );
            }
        }

        let mut state = HashToValues(vec![]);
        seed.hash(&mut state);
        InjectedContext::rng_seed(self, &state.0);
    }

    fn rng_save<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        InjectedContext::rng_save(self, ValueSerializer)
            .map_err(serde::ser::Error::custom)?
            .serialize(serializer)
    }

    fn rng_restore<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        InjectedContext::rng_restore(self, ValueDeserializer(Value::deserialize(deserializer)?))
            .map_err(serde::de::Error::custom)
    }

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
        InjectedContext::let_var(self, name, value);
    }

    fn var(&self, name: &Identifier) -> Option<&Value> {
        InjectedContext::var(self, name)
    }

    fn var_mut(&mut self, name: &Identifier) -> Option<&mut Value> {
        InjectedContext::var_mut(self, name)
    }

    fn inject(&mut self) -> &mut dyn InjectedContext {
        self
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Scope {
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
