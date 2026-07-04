use dices_values::{
    Injectable, Value, injectable,
    injected::call::InjectedContext,
    serde::{de::ValueDeserializer, error::Error as SerializationError, ser::ValueSerializer},
};

/// Rng bindings
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Rng {
    pub seed: Seed,
    pub save: Save,
    pub restore: Restore,
}

impl Rng {
    pub const fn new() -> Self {
        Self {
            seed: Seed,
            save: Save,
            restore: Restore,
        }
    }
}

/// Seed the rng
#[injectable]
pub fn Seed(#[cx] cx: &mut (impl InjectedContext + ?Sized), args: &[Value]) {
    cx.rng_seed(args);
}

/// Save the rng state
#[injectable]
pub fn Save(#[cx] cx: &mut (impl InjectedContext + ?Sized)) -> Result<Value, SerializationError> {
    cx.rng_save(ValueSerializer)
}

/// Restore the rng state
#[injectable]
pub fn Restore(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    state: Value,
) -> Result<(), SerializationError> {
    cx.rng_restore(ValueDeserializer(state))
}
