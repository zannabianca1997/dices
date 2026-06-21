use dices_ast::expr::Expr;
use dices_values::Value;

use crate::{EvalError, context::Context};

mod binary;
mod list;
mod map;
pub mod scope;
mod unary;
mod variable;

pub fn eval(expr: &Expr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    match expr {
        Expr::Const(value) => Ok(Value::clone(value)),
        Expr::Variable(ident) => variable::eval(ident, cx),
        Expr::Literal(literal) => super::literal::eval(literal, cx),
        Expr::List(list_expr) => list::eval(list_expr, cx),
        Expr::Map(map_expr) => map::eval(map_expr, cx),
        Expr::Binary(binary_expr) => binary::eval(binary_expr, cx),
        Expr::Unary(unary_expr) => unary::eval(unary_expr, cx),
        Expr::Scope(scope_expr) => scope::eval(scope_expr, cx),
    }
}
