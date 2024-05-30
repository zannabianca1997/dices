//! Intrisics of the language

use rand::Rng;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::{expr::EvalInterrupt, EvalContext, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, EnumIter)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(tag = "type", content = "value", deny_unknown_fields)
)]
pub enum Intrisic {
    Quit,
}

impl Intrisic {
    pub(crate) fn call(
        &self,
        params: Vec<Value>,
        context: &mut EvalContext<'_, '_, impl Rng>,
    ) -> Result<Value, EvalInterrupt> {
        match self {
            Intrisic::Quit => Err(if context.is_const() {
                EvalInterrupt::CannotEvalInConst("Cannot quit in consts")
            } else {
                EvalInterrupt::Quitted(params.into_boxed_slice())
            }),
        }
    }

    /// Build the library to expose to the user
    pub fn lib() -> Value {
        Value::Map(
            Self::iter()
                .map(|intr| (<&'static str>::from(intr).into(), Value::Intrisic(intr)))
                .collect(),
        )
    }
}
