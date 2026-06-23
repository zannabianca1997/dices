use std::{any::Any, error::Error};

use snafu::Snafu;

use crate::{
    Value,
    identifier::Identifier,
    injected::ValueInjected,
    int::ValueInt,
    serde::{de::ValueDeserializer, ser::ValueSerializer},
    string::ValueString,
};

/// Wrapped value is callable
pub trait Callable {
    fn call(&self, cx: &mut dyn InjectedContext, args: &[Value]) -> Result<Value, Box<dyn Error>>;
}

pub trait InjectedContext {
    /// Seed the random number generator
    fn rng_seed(&mut self, seed: &[Value]);

    /// Serialize the random number generator state
    fn rng_save(&self, serializer: ValueSerializer) -> crate::serde::error::Result<Value>;

    /// Restore the random number generator state
    fn rng_restore(&mut self, deserializer: ValueDeserializer) -> crate::serde::error::Result<()>;

    /// Throw a dice
    fn dice(&mut self, faces: ValueInt) -> ValueInt;

    /// Enter a scoped context
    ///
    /// The scoped context can read and set variables from the outside
    /// context, but not define new ones.
    fn enter_scope(&mut self) -> Box<dyn Any>;

    /// Exit the scoped context
    fn exit_scope(&mut self, data: Box<dyn Any>);

    /// Enter a jailed context
    ///
    /// The jailed context won't be able to modify or read any variable from the
    /// external scope.
    fn enter_jail(&mut self) -> Box<dyn Any>;

    /// Exit the jail
    fn exit_jail(&mut self, data: Box<dyn Any>);

    /// Create a variable
    ///
    /// If it exists in the current scope, shadows it
    fn let_var(&mut self, name: Identifier, value: Value);

    /// Get a variable value
    fn var(&self, name: &Identifier) -> Option<&Value>;

    /// Get a mutable variable value
    fn var_mut(&mut self, name: &Identifier) -> Option<&mut Value>;

    /// Get the standard library
    fn std(&self) -> ValueInjected;

    /// Print a value exactly like an expression result
    fn print(&self, value: Value);

    /// Display a manual page
    ///
    /// Returns only when the user exit the page
    fn manual(&self, page: ValueString) -> Result<(), ManualError>;

    /// Stop execution
    fn abort(&mut self, reason: Value) -> !;
}

#[derive(Debug, Snafu)]
pub enum ManualError {
    #[snafu(display("Unknown manual page {page}"))]
    UnknownPage { page: ValueString },
}
