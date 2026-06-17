use dices_ast::literal::Literal;
use dices_values::Value;

use crate::{EvalError, context::Context};

pub fn eval(literal: &Literal, _: &mut Context<'_>) -> Result<Value, EvalError> {
    Ok(match literal {
        Literal::Null(value) => value.clone().into(),
        Literal::Bool(value) => value.clone().into(),
        Literal::Int(value) => value.clone().into(),
        Literal::String(value) => value.clone().into(),
    })
}
