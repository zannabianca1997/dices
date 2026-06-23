//! Evaluation context

use std::error::Error;
use std::hash::{Hash, Hasher};
use std::mem;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::{collections::BTreeMap, iter::once};

use dices_ast::expr::scope::ScopeInner;
use dices_ast::identifier::Identifier;
use dices_std::Std;
use dices_values::Value;
use dices_values::injected::ValueInjected;
use dices_values::injected::call::{InjectedContext, ManualError};
use dices_values::injected::typed::TypedValueInjected;
use dices_values::int::ValueInt;
use dices_values::serde::de::ValueDeserializer;
use dices_values::serde::ser::ValueSerializer;
use dices_values::string::ValueString;
use num::FromPrimitive;
use num::traits::ConstOne;
use rand::Rng;
use rand_seeder::Seeder;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::ui::Ui;
use crate::{Engine, EvalError};

pub(crate) trait Context: Ui {
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

    fn as_injected(&mut self) -> &mut dyn InjectedContext;

    /// Get the standard library
    fn std(&self) -> TypedValueInjected<Std>;

    /// Stop execution
    fn abort(&mut self, reason: Value) -> !;
}

/// Evaluation context
pub struct EngineContext<'engine, Ui> {
    engine: &'engine mut Engine,
    scopes: Vec<Scope>,
    ui: Ui,
}

struct Abort {
    reason: Value,
}

impl<'engine, Ui> EngineContext<'engine, Ui> {
    /// Create a new context
    pub(crate) fn new(engine: &'engine mut Engine, ui: Ui) -> Self {
        Self {
            engine,
            scopes: vec![],
            ui,
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

    pub(crate) fn eval(&mut self, stmt: &ScopeInner) -> Result<Value, EvalError>
    where
        Ui: crate::ui::Ui,
    {
        match catch_unwind(AssertUnwindSafe(|| {
            crate::eval::expr::scope::eval_inner(stmt, self)
        })) {
            Ok(r) => r,
            Err(err) => match err.downcast::<Abort>() {
                Ok(abort) => Ok(abort.reason),
                Err(panic) => resume_unwind(panic),
            },
        }
    }
}

impl<'engine, Ui> Context for EngineContext<'engine, Ui>
where
    Ui: crate::ui::Ui,
{
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

    fn as_injected(&mut self) -> &mut dyn InjectedContext {
        self
    }

    fn std(&self) -> TypedValueInjected<Std> {
        self.engine.std.clone()
    }

    fn abort(&mut self, reason: Value) -> ! {
        #[cfg(not(panic = "unwind"))]
        compile_error!("Panic must be implemented via unwind to support abort");

        resume_unwind(Box::new(Abort { reason }))
    }
}

impl<U: Ui> Ui for EngineContext<'_, U> {
    type PrintError = U::PrintError;

    fn print(&self, value: impl Into<Value>) -> Result<(), Self::PrintError> {
        self.ui.print(value)
    }

    fn print_str<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError> {
        self.ui.print_str(value)
    }

    fn print_md<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError> {
        self.ui.print_md(value)
    }

    fn manual(&self, page: impl Into<ValueString>) -> Result<(), ManualError> {
        self.ui.manual(page)
    }
}

impl<U: Ui> InjectedContext for EngineContext<'_, U> {
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

    fn std(&self) -> ValueInjected {
        TypedValueInjected::type_erase(Context::std(self))
    }

    fn print(&self, value: Value) -> Result<(), Box<dyn Error>> {
        Ui::print(self, value).map_err(Into::into)
    }

    fn manual(&self, page: ValueString) -> Result<(), ManualError> {
        Ui::manual(self, page)
    }

    fn abort(&mut self, reason: Value) -> ! {
        Context::abort(self, reason)
    }

    fn print_str(&self, value: ValueString) -> Result<(), Box<dyn std::error::Error>> {
        Ui::print_str(self, value).map_err(Into::into)
    }

    fn print_md(&self, value: ValueString) -> Result<(), Box<dyn std::error::Error>> {
        Ui::print_md(self, value).map_err(Into::into)
    }
}

impl Context for dyn InjectedContext + '_ {
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

    fn as_injected(&mut self) -> &mut dyn InjectedContext {
        self
    }

    fn std(&self) -> TypedValueInjected<Std> {
        InjectedContext::std(self)
            .downcast()
            .expect("Only the standard library should be returned from `std`")
    }

    fn abort(&mut self, reason: Value) -> ! {
        InjectedContext::abort(self, reason)
    }
}

impl Ui for dyn InjectedContext + '_ {
    fn print(&self, value: impl Into<Value>) -> Result<(), Self::PrintError> {
        InjectedContext::print(self, value.into())
    }

    fn manual(&self, page: impl Into<ValueString>) -> Result<(), ManualError> {
        InjectedContext::manual(self, page.into())
    }

    type PrintError = Box<dyn Error>;

    fn print_str<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError> {
        InjectedContext::print_str(self, value.into())
    }

    fn print_md<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError> {
        InjectedContext::print_md(self, value.into())
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
