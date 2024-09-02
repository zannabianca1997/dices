//! This is the standard library of `dices`

use dices_ast::{
    intrisics::{InjectedIntr, Intrisic},
    values::ValueMap,
};

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
pub fn std<II>() -> ValueMap<II>
where
    II: InjectedIntr,
{
    std!(
            intrisics: Intrisic::all(),
            variadics: mod {
                call: Intrisic::Call,
                sum: Intrisic::Sum,
                join: Intrisic::Join,
                mult: Intrisic::Mult,
            },
            conversions: mod {
                to_number: Intrisic::ToNumber,
                to_list: Intrisic::ToList,
                to_string: Intrisic::ToString,
                parse: Intrisic::Parse,
            },
            prelude: mod {
                sum: Intrisic::Sum,
                join: Intrisic::Join,
                mult: Intrisic::Mult,

                to_number: Intrisic::ToNumber,
                to_list: Intrisic::ToList,
                to_string: Intrisic::ToString,
                parse: Intrisic::Parse,
            }
    )
}
