use dices_std::Std;
use dices_values::Value;

use crate::{EvalError, context::Context, var_use::VarUse};

pub fn eval(_cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    Ok(Std::new().into())
}

pub fn var_use() -> VarUse {
    VarUse::none()
}
