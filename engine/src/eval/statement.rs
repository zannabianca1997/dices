use dices_ast::statement::Statement;

use crate::{EvalError, context::Context};

mod assign;

pub fn eval(stmt: &Statement, cx: &mut Context<'_>) -> Result<(), EvalError> {
    match stmt {
        Statement::Assign(assign) => assign::eval(assign, cx),
        Statement::Expr(expr) => super::expr::eval(expr, cx).map(|_| ()),
        Statement::Empty => Ok(()),
    }
}
