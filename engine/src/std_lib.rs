//! Initialize the `dices` std lib

use std::rc::Rc;

use crate::{
    identifier::{self, IdentStr},
    intrisics::Intrisic,
    Value,
};

pub fn std_lib() -> Value {
    Value::Map(
        [
            ("intrisics".into(), Intrisic::lib()),
            (
                "prelude".into(),
                Value::Map(
                    prelude()
                        .into_iter()
                        .map(|(k, v)| (identifier::from_rc(k), v))
                        .collect(),
                ),
            ),
        ]
        .into(),
    )
}

pub fn prelude() -> impl IntoIterator<Item = (Rc<IdentStr>, Value)> {
    [(
        IdentStr::new("quit").unwrap().into(),
        Value::Intrisic(Intrisic::Quit),
    )]
}
