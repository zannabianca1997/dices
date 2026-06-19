use dices_ast::statement::Statement;
use dices_values::{Value, null::ValueNull};

use crate::{EvalError, context::Context};

pub fn eval(stmt: &Statement, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    match stmt {
        Statement::Expr(expr) => super::expr::eval(expr, cx),
        Statement::Empty => Ok(ValueNull.into()),
    }
}
