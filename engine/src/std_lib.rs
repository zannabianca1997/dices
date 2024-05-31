//! Initialize the `dices` std lib

use std::rc::Rc;

use crate::{
    identifier::{self, IdentStr},
    intrisics::Intrisic,
    Printer, Value,
};

pub fn std_lib<RNG, P: Printer>() -> Value {
    Value::Map(
        [
            ("intrisics".into(), Intrisic::lib::<RNG, P>()),
            (
                "prelude".into(),
                Value::Map(
                    prelude::<RNG, P>()
                        .into_iter()
                        .map(|(k, v)| (identifier::from_rc(k), v))
                        .collect(),
                ),
            ),
        ]
        .into(),
    )
}

pub fn prelude<RNG, P: Printer>() -> impl IntoIterator<Item = (Rc<IdentStr>, Value)> {
    let mut prelude = vec![];
    prelude.push((
        IdentStr::new("quit").unwrap().into(),
        Value::Intrisic(Intrisic::Quit),
    ));
    prelude.push((
        IdentStr::new("help").unwrap().into(),
        Value::Intrisic(Intrisic::Help),
    ));
    if Intrisic::Print.available::<RNG, P>() {
        prelude.push((
            IdentStr::new("print").unwrap().into(),
            Value::Intrisic(Intrisic::Print),
        ))
    }
    prelude
}
