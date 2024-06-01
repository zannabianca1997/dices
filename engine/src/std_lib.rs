//! Initialize the `dices` std lib

use std::rc::Rc;

use crate::{
    identifier::{self, IdentStr},
    intrisics::Intrisic,
    Callbacks, Value,
};

pub fn std_lib<RNG, C: Callbacks>() -> Value {
    Value::Map(
        [
            ("intrisics".into(), Intrisic::lib::<RNG, C>()),
            (
                "prelude".into(),
                Value::Map(
                    prelude::<RNG, C>()
                        .into_iter()
                        .map(|(k, v)| (identifier::from_rc(k), v))
                        .collect(),
                ),
            ),
        ]
        .into(),
    )
}

pub fn prelude<RNG, C: Callbacks>() -> impl IntoIterator<Item = (Rc<IdentStr>, Value)> {
    let mut prelude = vec![];
    prelude.push((
        IdentStr::new("quit").unwrap().into(),
        Value::Intrisic(Intrisic::Quit),
    ));
    if Intrisic::Print.available::<RNG, C>() {
        prelude.push((
            IdentStr::new("print").unwrap().into(),
            Value::Intrisic(Intrisic::Print),
        ))
    }
    if Intrisic::Help.available::<RNG, C>() {
        prelude.push((
            IdentStr::new("help").unwrap().into(),
            Value::Intrisic(Intrisic::Help),
        ))
    }
    prelude
}
