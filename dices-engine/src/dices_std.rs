//! This is the standard library of `dices`

use dices_ast::{intrisics::Intrisic, values::ValueMap};

/// Build the default std library
pub fn std() -> ValueMap {
    // Build the intrisics
    let intrisics = Intrisic::module();
    // Build the prelude
    let prelude = ValueMap::from_iter([]);
    // Build the module
    ValueMap::from_iter([
        ("intrisics".into(), intrisics.into()),
        ("prelude".into(), prelude.into()),
    ])
}
