//! Intrisics of the language

use rand::Rng;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::{expr::EvalInterrupt, Callbacks, EvalContext, EvalError, Printer, Value};

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
        context: &mut EvalContext<'_, '_, impl Rng, impl Printer>,
    ) -> Result<Value, EvalInterrupt> {
        match self {
            Intrisic::Quit => quit(context, params),
            Intrisic::Help => help(params),
            Intrisic::Print => print(context, params),
        }
    }

    /// Build the library to expose to the user
    pub(crate) fn lib<RNG, P: Printer>() -> Value {
        Value::Map(
            Self::iter()
                .map(|intr| {
                    (
                        <&'static str>::from(intr).into(),
                        if intr.available::<RNG, P>() {
                            Value::Intrisic(intr)
                        } else {
                            Value::Null
                        },
                    )
                })
                .collect(),
        )
    }

    pub(crate) fn available<RNG, P: Printer>(&self) -> bool {
        match self {
            Intrisic::Quit => true,
            Intrisic::Print => P::AVAILABLE,
            Intrisic::Help => true,
        }
    }
}
mod man {
    pub(super) fn man(topic: &str) -> Option<&'static str> {
        todo!()
    }
}

fn help(params: Vec<Value>) -> Result<Value, EvalInterrupt> {
    todo!()
}

fn quit<R, P>(context: &mut EvalContext<R, P>, params: Vec<Value>) -> Result<Value, EvalInterrupt> {
    Err(if context.is_const() {
        EvalInterrupt::CannotEvalInConst("quit")
    } else {
        EvalInterrupt::Quitted(params.into_boxed_slice())
    })
}

fn print<R, P: Printer>(
    context: &mut EvalContext<R, P>,
    params: Vec<Value>,
) -> Result<Value, EvalInterrupt> {
    match context {
        EvalContext::Engine {
            callbacks: Callbacks { print, .. },
            ..
        } => {
            if P::AVAILABLE {
                print.print(match params.len() {
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
            } else {
                unreachable!("print intrisic should be unreachable if printer is not available")
            }
        }
        EvalContext::Const { .. } => Err(EvalInterrupt::CannotEvalInConst("print")),
    }
}
