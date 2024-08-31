//! This is the standard library of `dices`

use dices_ast::{intrisics::Intrisic, values::ValueMap};

macro_rules! map {
    (
        @ACC [$($acc:tt)*]
    ) => {
        ValueMap::from_iter([$($acc)*])
    };
    (
        @ACC [$($acc:tt)*] $name:ident: $value:expr $(, $($rest:tt)*)?
    ) => {
        map!(
            @ACC [$($acc)* (stringify!($name).into(), ($value).into()),] $($($rest)*)?
        )
    };
    (
        @ACC [$($acc:tt)*] $name:ident: mod { $($inner:tt)* } $(, $($rest:tt)*)?
    ) => {
        map!(
            @ACC [$($acc)* (stringify!($name).into(), map!(@ACC [] $($inner)*).into()),] $($($rest)*)?
        )
    };
}

macro_rules! std {
    ($($tokens:tt)*) => {map!(@ACC [] $($tokens)*)};
}

/// Build the default std library
pub fn std() -> ValueMap {
    std!(
            intrisics: Intrisic::all(),
            filters: mod {
                kh: Intrisic::KeepHigh,
                kl: Intrisic::KeepLow,
                rh: Intrisic::RemoveHigh,
                rl: Intrisic::RemoveLow
            },
            prelude: mod {}
    )
}
