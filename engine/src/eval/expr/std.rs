use dices_values::Value;

use crate::{EvalError, context::Context, var_use::VarUse};

pub fn eval(cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    Ok(cx.std().into())
}

pub fn var_use() -> VarUse {
    VarUse::none()
}
