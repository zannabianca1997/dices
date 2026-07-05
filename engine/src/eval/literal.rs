use dices_ast::literal::Literal;
use dices_values::Value;

use crate::{EvalError, context::Context, var_use::VarUse};

pub fn eval(literal: &Literal, _: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    Ok(match literal {
        Literal::Null(value) => value.0.into(),
        Literal::Bool(value) => value.0.into(),
        Literal::Int(value) => value.0.clone().into(),
        Literal::String(value) => value.0.clone().into(),
    })
}

pub(crate) fn var_use(_: &Literal) -> VarUse {
    VarUse::none()
}
