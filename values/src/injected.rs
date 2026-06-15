use std::{
    error::Error,
    fmt::{self, Debug, Formatter},
    sync::Arc,
};

use serde::Serialize;

use crate::Value;

#[derive(Debug, Clone)]
pub struct ValueInjected<'a>(Arc<dyn Injected + 'a>);

impl<'a> ValueInjected<'a> {
    /// Create a new injected value
    pub fn new(injected: impl Injected + 'a) -> Self {
        Self(Arc::new(injected))
    }
}

/// A wrapped value that can be interacted with from the repl
pub trait Injected: Debug  {
    /// Full path to the type (e.g. `my-crate::path::to::Type`)
    fn type_path(&self) -> &'static str;

    /// Human readable description of this object
    fn description(&self, f: Formatter<'_>) -> fmt::Result;

    /// The object has a representation in dices value
    fn as_readable(&self) -> Option<&dyn Readable> {
        None
    }
    /// The object can be written to with a dices value
    fn as_writable(&self) -> Option<&dyn Writable> {
        None
    }
    /// The object can be called
    fn as_callable(&self) -> Option<&dyn Callable> {
        None
    }
}

/// Wrapped value is readable as a dices value
pub trait Readable {
    fn read(&self) -> Result<Value, Box<dyn Error>>;
}
/// Wrapped value is writable with a dices value
pub trait Writable {
    fn write(&self, value: Value) -> Result<(), Box<dyn Error>>;
}
/// Wrapped value is callable
pub trait Callable {
    fn call(&self, args: &[Value]) -> Result<Value, Box<dyn Error>>;
}


/// Inject an object using its `Serialize` implementation
pub trait ReadableWithSerde: Serialize + Debug  {
    /// Full path to the type (e.g. `my-crate::path::to::Type`)
    fn type_path(&self) -> &'static str;
    /// Human readable description of this object
    fn description(&self, f: Formatter<'_>) -> fmt::Result;
}

impl<T> Injected for T
where
    T: ReadableWithSerde,
{
    fn type_path(&self) -> &'static str {
        <T as ReadableWithSerde>::type_path(&self)
    }
    fn description(&self, f: Formatter<'_>) -> fmt::Result {
        <T as ReadableWithSerde>::description(&self, f)
    }
    fn as_readable(&self) -> Option<&dyn Readable> {
        Some(self)
    }
}

impl<T> Readable for T
where
    T: ReadableWithSerde,
{
    fn read(&self) -> Result<Value, Box<dyn Error>> {
        crate::serde::to_value(self).map_err(|err| Box::new(err) as Box<dyn Error>)
    }
}
