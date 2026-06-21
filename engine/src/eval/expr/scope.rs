use dices_ast::expr::scope::{ScopeExpr, ScopeInner};
use dices_values::{Value, null::ValueNull};

use crate::{EvalError, context::Context, var_use::VarUse};

pub(super) fn eval(expr: &ScopeExpr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    cx.scope(|cx| eval_inner(&expr.0, cx))
}

pub fn eval_inner(expr: &ScopeInner, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    for stmt in &expr.statements {
        crate::eval::statement::eval(&stmt, cx)?;
    }

    if let Some(expr) = &expr.expr {
        super::eval(expr, cx)
    } else {
        Ok(ValueNull.into())
    }
}

pub(crate) fn var_use(expr: &ScopeExpr) -> VarUse {
    VarUse::sequence(
        expr.0
            .statements
            .iter()
            .map(super::super::statement::var_use)
            .chain(expr.0.expr.as_ref().map(super::var_use)),
    )
    .scoped()
}
