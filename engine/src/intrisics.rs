//! Intrisics of the language

use man::man;
use rand::Rng;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::{expr::EvalInterrupt, Callbacks, EvalContext, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, EnumIter)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(tag = "type", content = "value", deny_unknown_fields)
)]
pub enum Intrisic {
    Quit,
    Print,
    Help,
}

impl Intrisic {
    pub(crate) fn call(
        &self,
        params: Vec<Value>,
        context: &mut EvalContext<'_, '_, impl Rng, impl Callbacks>,
    ) -> Result<Value, EvalInterrupt> {
        match self {
            Intrisic::Quit => quit(context, params),
            Intrisic::Help => help(context, params),
            Intrisic::Print => print(context, params),
        }
    }

    /// Build the library to expose to the user
    pub(crate) fn lib<RNG, C: Callbacks>() -> Value {
        Value::Map(
            Self::iter()
                .map(|intr| {
                    (
                        <&'static str>::from(intr).to_lowercase().into(),
                        if intr.available::<RNG, C>() {
                            Value::Intrisic(intr)
                        } else {
                            Value::Null
                        },
                    )
                })
                .collect(),
        )
    }

    pub(crate) fn available<RNG, C: Callbacks>(&self) -> bool {
        match self {
            Intrisic::Quit => true,
            Intrisic::Print => C::PRINT_AVAIL,
            Intrisic::Help => C::HELP_AVAIL,
        }
    }
}

fn help<R, C: Callbacks>(
    context: &mut EvalContext<R, C>,
    params: Vec<Value>,
) -> Result<Value, EvalInterrupt> {
    match context {
        EvalContext::Engine { callbacks, .. } => {
            debug_assert!(C::HELP_AVAIL);
            let topic = match &*params {
                [] => "introduction",
                [Value::String(s)] => &**s,
                // help do not throw any errors, at most it print the general help page
                _ => "intrisics/help",
            };
            let page = man(topic)
                .unwrap_or_else(|| man("index").expect("The index should always be generated"));

            callbacks.help(&page.content);
            Ok(Value::Null)
        }
        EvalContext::Const { .. } => Err(EvalInterrupt::CannotEvalInConst("help")),
    }
}

fn quit<R, C>(context: &mut EvalContext<R, C>, params: Vec<Value>) -> Result<Value, EvalInterrupt> {
    Err(if context.is_const() {
        EvalInterrupt::CannotEvalInConst("quit")
    } else {
        EvalInterrupt::Quitted(params.into_boxed_slice())
    })
}

fn print<R, C: Callbacks>(
    context: &mut EvalContext<R, C>,
    params: Vec<Value>,
) -> Result<Value, EvalInterrupt> {
    match context {
        EvalContext::Engine { callbacks, .. } => {
            debug_assert!(C::PRINT_AVAIL);
            callbacks.print(match params.len() {
                0 => Value::Null,
                1 => {
                    let [v] = Box::into_inner(
                        Box::<[_; 1]>::try_from(params.into_boxed_slice()).unwrap(),
                    );
                    v
                }
                _ => Value::List(params),
            });
            Ok(Value::Null)
        }
        EvalContext::Const { .. } => Err(EvalInterrupt::CannotEvalInConst("print")),
    }
}
