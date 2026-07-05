use dices_values::{
    Injectable, Value, injectable,
    injected::call::InjectedContext,
    serde::{de::ValueDeserializer, error::Error as SerializationError, ser::ValueSerializer},
};

/// 5.2. Rng
///
/// Controls of the random number generator.
///
/// `dices` works with a global RNG, seeded at the start of the session. Here
/// are functions to control it, reseed it and save and store the result.
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

/// 5.2.1. Seed
///
/// Seed the rng with the hash of the arguments. After seeding the rng behavior
/// is fully predictable from the seed value.
///
/// ```dices
/// #>> let seed = std.rng.seed;
/// >>> seed(42)
/// >>> 2d6
/// [3, 5]
/// >>> seed(42)
/// >>> 2d6
/// [3, 5]
/// ```
#[injectable]
pub fn Seed(#[cx] cx: &mut (impl InjectedContext + ?Sized), args: &[Value]) {
    cx.rng_seed(args);
}

/// 5.2.2. Save
///
/// Save the rng state, to be restored later with `restore`
///
/// ```dices
/// #>> let seed = std.rng.seed;
/// #>> let save = std.rng.save;
/// >>> seed(42)
/// >>> save()
/// _
/// ```
#[injectable]
pub fn Save(#[cx] cx: &mut (impl InjectedContext + ?Sized)) -> Result<Value, SerializationError> {
    cx.rng_save(ValueSerializer)
}

/// 5.2.3. Restore
///
/// Restore the rng state obtained from a previous call to `save`
///
/// ```dices
/// #>> let seed = std.rng.seed;
/// #>> let save = std.rng.save;
/// #>> let restore = std.rng.restore;
/// >>> seed(42)
/// >>> let state = save()
/// >>> let first = 1d6
/// >>> restore(state)
/// >>> let second = 1d6
/// >>> [first, second]
/// [[3], [3]]
/// ```
#[injectable]
pub fn Restore(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    state: Value,
) -> Result<(), SerializationError> {
    cx.rng_restore(ValueDeserializer(state))
}
