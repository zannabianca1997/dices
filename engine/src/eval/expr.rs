use dices_ast::expr::Expr;
use dices_values::Value;

use crate::{EvalError, context::Context};

mod binary;
pub mod scope;
mod unary;

pub fn eval(expr: &Expr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    match expr {
        Expr::Const(value) => Ok(Value::clone(value)),
        Expr::Literal(literal) => super::literal::eval(literal, cx),
        Expr::Binary(binary_expr) => binary::eval(binary_expr, cx),
        Expr::Unary(unary_expr) => unary::eval(unary_expr, cx),
        Expr::Scope(scope_expr) => scope::eval(scope_expr, cx),
    }
}
