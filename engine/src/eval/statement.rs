use dices_ast::statement::Statement;

use crate::{EvalError, context::Context};

pub fn eval(stmt: &Statement, cx: &mut Context<'_>) -> Result<(), EvalError> {
    match stmt {
        Statement::Expr(expr) => super::expr::eval(expr, cx).map(|_| ()),
        Statement::Empty => Ok(()),
    }
}
