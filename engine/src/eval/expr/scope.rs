use dices_ast::expr::scope::{ScopeExpr, ScopeInner};
use dices_values::{Value, null::ValueNull};

use crate::{EvalError, context::Context};

pub(super) fn eval(expr: &ScopeExpr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    cx.scope(|cx| eval_inner(&expr.0, cx))
}

pub fn eval_inner(expr: &ScopeInner, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    for stmt in &expr.statements {
        crate::eval::statement::eval(&stmt, cx)?;
    }

    if let Some(expr) = &expr.expr {
        super::eval(expr, cx)
    } else {
        Ok(ValueNull.into())
    }
}
