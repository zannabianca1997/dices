use std::{any::Any, error::Error, fmt::Debug, sync::Arc};

use serde::Serialize;

use crate::Value;

#[derive(Debug)]
pub struct ValueInjected(Arc<dyn Injected>);

impl ValueInjected {
    /// Create a new injected value
    pub fn new(injected: impl Injected) -> Self {
        Self(Arc::new(injected))
    }

    /// Downcast the value as the original type
    pub fn dowcast_ref<T: Injected>(&self) -> Option<&T> {
        (&*self.0 as &dyn Any).downcast_ref()
    }
}

/// A wrapped value that can be interacted with from the repl
pub trait Injected: Any + Debug + DynEq + 'static {
    /// Human readable description of this object
    fn human_description(&self) -> &str;
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

impl PartialEq for ValueInjected {
    fn eq(&self, other: &Self) -> bool {
        &*self.0 == &*other.0
    }
}
impl Eq for ValueInjected {}

// === Comparisons ===
//
// Need a small dance to correctly downcast, with two dynamic dispatches Result
// is that two values are equal iff their type is the same and they compare
// equal

pub trait DynEq: Any + DynEqRhs {
    fn as_any(&self) -> &dyn Any;
    fn as_rhs(&self) -> &dyn DynEqRhs;
    fn dispatch_lhs(&self, arg2: &dyn DynEqRhs) -> bool;

    fn dyn_eq<T: PartialEq + 'static>(&self, other: &T) -> bool
    where
        Self: Sized,
    {
        if let Some(this) = self.as_any().downcast_ref::<T>() {
            this == other
        } else {
            false
        }
    }
}

pub trait DynEqRhs {
    fn dispatch_rhs(&self, arg1: &dyn DynEq) -> bool;
}

impl<T: Any + Eq> DynEq for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_rhs(&self) -> &dyn DynEqRhs {
        self
    }

    fn dispatch_lhs(&self, arg2: &dyn DynEqRhs) -> bool {
        arg2.dispatch_rhs(self)
    }
}

impl<T: Any + Eq> DynEqRhs for T {
    fn dispatch_rhs(&self, arg1: &dyn DynEq) -> bool {
        if let Some(other) = arg1.as_any().downcast_ref::<Self>() {
            other.dyn_eq(self)
        } else {
            false
        }
    }
}

impl PartialEq for dyn Injected {
    fn eq(&self, other: &Self) -> bool {
        self.dispatch_lhs(other.as_rhs())
    }
}
impl Eq for dyn Injected {}

/// Inject an object using its `Serialize` implementation
pub trait InjectedWithSerde: Serialize + Debug + DynEq + 'static {
    /// Human readable description of this object
    fn human_description(&self) -> &str;
}

impl<T> Injected for T
where
    T: InjectedWithSerde,
{
    fn human_description(&self) -> &str {
        <T as InjectedWithSerde>::human_description(&self)
    }
}

impl<T> Readable for T
where
    T: InjectedWithSerde,
{
    fn read(&self) -> Result<Value, Box<dyn Error>> {
        crate::serde::to_value(self).map_err(|err| Box::new(err) as Box<dyn Error>)
    }
}
