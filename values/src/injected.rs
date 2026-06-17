use std::{
    cmp::Ordering,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    sync::Arc,
};

use snafu::{OptionExt, Snafu};
use yoke::Yoke;

use crate::{Value, injected::read::ValueOrInject};
use describable::Description;
use read::ReadValue;
use required_traits::RequiredTraits;

pub mod describable;
pub mod read;
mod required_traits;

/// A wrapped value that can be interacted with from the repl
#[derive(Debug, Clone)]
pub struct ValueInjected(Yoke<&'static dyn Injectable, Option<Arc<Box<dyn Injectable>>>>);

/// A value that can be wrapped and passed to the repl
pub trait Injectable: RequiredTraits {
    /// The object has a representation in dices value
    ///
    /// When this object is used in operation it will be read, and the result
    /// will be used in its place
    fn as_readable(&self) -> Option<&dyn read::Readable> {
        None
    }
}

impl ValueInjected {
    /// Create a new injected value
    pub fn new(injected: impl Injectable) -> Self {
        Self(
            Yoke::attach_to_cart(
                Arc::new(Box::new(injected) as Box<dyn Injectable>),
                |cart| &**cart,
            )
            .wrap_cart_in_option(),
        )
    }

    /// Create a new value from a static implementor
    pub const fn new_static(injected: &'static impl Injectable) -> Self {
        Self(Yoke::new_owned(injected as &dyn Injectable))
    }

    /// Description of this value
    pub fn description(&self) -> Description<'_> {
        Description(*self.0.get())
    }

    /// Check if this value is readable as a dices value
    pub fn is_readable(&self) -> bool {
        self.0.get().as_readable().is_some()
    }

    /// Read this value as a dices value
    pub fn read(&self) -> Result<Value, ReadError> {
        let readable = self.0.get().as_readable().context(NotReadableSnafu)?;
        let value = readable.read()?;
        Ok(unsafe {
            // Safety: `as_readable` can borrow only from the initial object
            self.attach_read_value_to_cart(value)
        })
    }

    /// Safety: `self` must be borrowing _only_ from the cart allocation
    unsafe fn attach_read_value_to_cart<'a>(&'a self, value: ReadValue<'a>) -> Value {
        match value {
            ReadValue::Value(value) => value,
            ReadValue::Map(map) => Value::Map(
                map.into_iter()
                    .map(|(k, v)| (k, unsafe { self.attach_value_or_inject_to_cart(v) }))
                    .collect(),
            ),
            ReadValue::List(list) => Value::List(
                list.into_iter()
                    .map(|v| unsafe { self.attach_value_or_inject_to_cart(v) })
                    .collect(),
            ),
        }
    }
    /// Safety: `self` must be borrowing _only_ from the cart allocation
    unsafe fn attach_value_or_inject_to_cart<'a>(&'a self, value: ValueOrInject<'a>) -> Value {
        match value {
            ValueOrInject::Inject(injected) => {
                let attached = self.0.map_project_cloned(|_, _| unsafe {
                    // Safety: we are extending the lifetime to 'static, but
                    // also wrapping into a `Yoke` that will stop any longer
                    // than ok reference to be emitted.
                    &*(injected as *const dyn Injectable)
                });
                Value::Injected(ValueInjected(attached))
            }
            ValueOrInject::Value(value) => unsafe { self.attach_read_value_to_cart(value) },
        }
    }
}

pub trait Inject: Injectable + Sized {
    fn into_value(self) -> ValueInjected {
        ValueInjected::new(self)
    }

    fn static_into_value(&'static self) -> ValueInjected {
        ValueInjected::new_static(self)
    }
}
impl<T: Injectable> Inject for T {}

impl PartialEq for ValueInjected {
    fn eq(&self, other: &Self) -> bool {
        self.0.get().dyn_eq(*other.0.get())
    }
}
impl Eq for ValueInjected {}

impl Display for ValueInjected {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.description())
    }
}

impl PartialOrd for ValueInjected {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueInjected {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.get().dyn_cmp(*other.0.get())
    }
}

impl Hash for ValueInjected {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.get().dyn_hash(state);
    }
}

#[derive(Debug, Snafu)]
pub enum ReadError {
    NotReadable,
    #[snafu(transparent)]
    ReadFailed {
        source: Box<dyn Error>,
    },
}
