use std::{
    any::Any,
    cmp::Ordering,
    collections::BTreeMap,
    error::Error,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    sync::Arc,
};

use serde::Serialize;
use snafu::{OptionExt, Snafu};
use yoke::Yoke;

use crate::{Value, injected::sealed::Sealed, string::ValueString};

#[derive(Debug, Clone)]
pub struct ValueInjected(Yoke<&'static dyn Injected, Arc<dyn Injected>>);

impl ValueInjected {
    /// Create a new injected value
    pub fn new(injected: impl Injected + 'static) -> Self {
        Self(Yoke::attach_to_cart(Arc::new(injected), |cart| &*cart))
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
        let read = readable.read()?;
        Ok(unsafe {
            // Safety: `as_readable` can borrow only from the initial object
            read.attach_to_cart(self.0.backing_cart())
        })
    }
}

impl<'a> ReadValue<'a> {
    /// Safety: `self` must be borrowing _only_ from the cart allocation
    unsafe fn attach_to_cart(self, cart: &'a Arc<dyn Injected>) -> Value {
        match self {
            ReadValue::Value(value) => value,
            ReadValue::Map(map) => Value::Map(
                map.into_iter()
                    .map(|(k, v)| (k, unsafe { v.attach_to_cart(cart) }))
                    .collect(),
            ),
            ReadValue::List(list) => Value::List(
                list.into_iter()
                    .map(|v| unsafe { v.attach_to_cart(cart) })
                    .collect(),
            ),
            ReadValue::Inject(injected) => {
                let cart = Arc::clone(cart);
                let var_name = injected as *const dyn Injected;
                let new_always_owned: Yoke<&'static dyn Injected, Arc<dyn Injected>> =
                    Yoke::attach_to_cart(cart, |_| unsafe { &*var_name });
                let value = ValueInjected(new_always_owned);

                Value::Injected(value)
            }
        }
    }
}

impl PartialEq for ValueInjected {
    fn eq(&self, other: &Self) -> bool {
        self.0.get().dyn_eq(other.0.get().as_dyn_traits())
    }
}
impl Eq for ValueInjected {}

pub struct Description<'a>(&'a dyn Injected);

impl Display for Description<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.description(f)
    }
}

impl Display for ValueInjected {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.description())
    }
}

impl PartialOrd for ValueInjected {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.get().dyn_partial_cmp(other.0.get().as_dyn_traits())
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

pub trait Describable {
    /// Human readable description of this object
    fn description(&self, f: &mut Formatter<'_>) -> fmt::Result;
}

/// A wrapped value that can be interacted with from the repl
pub trait Injected: Debug + DynTraits + Describable {
    /// The object has a representation in dices value
    fn as_readable(&self) -> Option<&dyn Readable> {
        None
    }
}

pub enum ReadValue<'a> {
    Value(Value),
    Map(BTreeMap<ValueString, ReadValue<'a>>),
    List(Vec<ReadValue<'a>>),
    Inject(&'a dyn Injected),
}

/// Wrapped value is readable as a dices value
pub trait Readable {
    fn read(&self) -> Result<ReadValue<'_>, Box<dyn Error>>;
}

/// Implement [`Readable`] on an object using its [`Serialize`] implementation
pub trait ReadableWithSerde: Serialize {}

impl<T> Readable for T
where
    T: ReadableWithSerde,
{
    fn read(&self) -> Result<ReadValue<'_>, Box<dyn Error>> {
        crate::serde::to_value(self)
            .map(ReadValue::Value)
            .map_err(|err| Box::new(err) as Box<dyn Error>)
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Implementations of common traits in a dynamic dispatch friendly way
pub trait DynTraits: Any + Sealed {
    fn as_any(&self) -> &dyn Any;
    fn as_dyn_traits(&self) -> &dyn DynTraits;

    fn dyn_eq(&self, other: &dyn DynTraits) -> bool;
    fn dyn_partial_cmp(&self, other: &dyn DynTraits) -> Option<Ordering>;
    fn dyn_hash(&self, state: &mut dyn Hasher);
}
impl<T> Sealed for T where T: Any + Eq + PartialOrd + Hash {}
impl<T> DynTraits for T
where
    T: Any + Eq + PartialOrd + Hash,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_dyn_traits(&self) -> &dyn DynTraits {
        self
    }
    fn dyn_eq(&self, other: &dyn DynTraits) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self == other
        } else {
            false
        }
    }

    fn dyn_partial_cmp(&self, other: &dyn DynTraits) -> Option<Ordering> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.partial_cmp(other)
        } else {
            None
        }
    }

    fn dyn_hash(&self, state: &mut dyn Hasher) {
        struct Wrapper<'a>(&'a mut dyn Hasher);

        macro_rules! pass {
            ( $($ty:ident)* ) => {
                paste::paste!{
                    $(
                        fn [< write_ $ty >] ( &mut self, i: $ty ) {
                            self.0. [< write_ $ty >] (i)
                        }
                    )*
                }
            };
        }

        impl Hasher for Wrapper<'_> {
            fn finish(&self) -> u64 {
                self.0.finish()
            }

            fn write(&mut self, bytes: &[u8]) {
                self.0.write(bytes);
            }

            pass! {
                u8 u16 u32 u64 u128 usize
                i8 i16 i32 i64 i128 isize
            }
        }

        self.hash(&mut Wrapper(state));
    }
}
