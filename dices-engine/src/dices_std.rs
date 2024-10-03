//! This is the standard library of `dices`

use dices_ast::{
    intrisics::{InjectedIntr, Intrisic},
    value::{Value, ValueMap},
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

fn version_value<II>() -> Value<II> {
    dices_ast::value::serde::serialize_to_value(dices_ast::version::VERSION)
        .expect("Version should be serializable to a value")
}

/// Build the default std library
pub fn std<II>() -> ValueMap<II>
where
    II: InjectedIntr,
{
    let mut dices_std = std!(
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
            },
            versions: mod {
                ast: version_value()
            }
    );
    #[cfg(feature = "json")]
    {
        let conversions = dices_std
            .get_mut("conversions")
            .unwrap()
            .as_map_mut()
            .unwrap();
        conversions.insert("to_json".into(), Intrisic::ToJson.into());
        conversions.insert("from_json".into(), Intrisic::FromJson.into());
    }
    // injecting the injected intrisics in the required places
    for intrisic in II::iter() {
        for path in intrisic.std_paths() {
            let mut path_parts = path.iter();
            let name = path_parts.next_back().expect(
                "The injected intrisics should have at least one path component (the name)",
            );
            let mut map = &mut dices_std;
            for part in path_parts {
                if !map.contains(part) {
                    map.insert((&**part).into(), ValueMap::new().into());
                }
                map = match map.get_mut(&*part).unwrap() {
                    Value::Map(map) => map,
                    _ => panic!("Clash in injecting the intrisics in the std library"),
                }
            }
            map.insert(
                (&**name).into(),
                Value::Intrisic(Intrisic::Injected(intrisic.clone()).into()),
            );
        }
    }
    dices_std
}
