use dices_ast::literal::Literal;
use dices_values::Value;

use crate::{EvalError, context::Context};

pub fn eval(literal: &Literal, _: &mut Context<'_>) -> Result<Value, EvalError> {
    Ok(match literal {
        Literal::Null(value) => value.0.into(),
        Literal::Bool(value) => value.0.into(),
        Literal::Int(value) => value.0.clone().into(),
        Literal::String(value) => value.0.clone().into(),
    })
}
