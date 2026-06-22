use dices_ast::expr::Expr;
use dices_values::Value;

use crate::{EvalError, context::Context, var_use::VarUse};

mod binary;
mod call;
mod closure;
mod list;
mod map;
pub mod scope;
mod unary;
mod variable;
mod std;

pub fn eval(expr: &Expr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    match expr {
        Expr::Const(value) => Ok(Value::clone(value)),
        Expr::Variable(ident) => variable::eval(ident, cx),
        Expr::Literal(literal) => super::literal::eval(literal, cx),
        Expr::List(list_expr) => list::eval(list_expr, cx),
        Expr::Map(map_expr) => map::eval(map_expr, cx),
        Expr::Binary(binary_expr) => binary::eval(binary_expr, cx),
        Expr::Unary(unary_expr) => unary::eval(unary_expr, cx),
        Expr::Scope(scope_expr) => scope::eval(scope_expr, cx),
        Expr::Closure(closure_expr) => closure::eval(closure_expr, cx),
        Expr::Call(call_expr) => call::eval(call_expr, cx),
        Expr::Std => std::eval(cx),
    }
}

pub fn var_use(expr: &Expr) -> VarUse {
    match expr {
        Expr::Const(_) => VarUse::none(),
        Expr::Variable(identifier) => variable::var_use(identifier),
        Expr::Literal(literal) => super::literal::var_use(literal),
        Expr::List(list_expr) => list::var_use(list_expr),
        Expr::Map(map_expr) => map::var_use(map_expr),
        Expr::Binary(binary_expr) => binary::var_use(binary_expr),
        Expr::Unary(unary_expr) => unary::var_use(unary_expr),
        Expr::Scope(scope_expr) => scope::var_use(scope_expr),
        Expr::Closure(closure_expr) => closure::var_use(closure_expr),
        Expr::Call(call_expr) => call::var_use(call_expr),
        Expr::Std => std::var_use(),
    }
}
