use std::iter::once;

use dices_ast::expr::call::CallExpr;
use dices_values::{Value, injected::CallError};
use itertools::Itertools;
use snafu::ResultExt;

use crate::{CallSnafu, EvalError, context::Context, var_use::VarUse};

pub(crate) fn eval(expr: &CallExpr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    let called = super::eval(&expr.called, cx)?;

    let callable = called
        .clone()
        .try_unwrap_injected()
        .map_err(|err| err.input)
        .and_then(|c| {
            if c.is_callable() {
                Ok(c)
            } else {
                Err(c.into())
            }
        })
        .map_err(|_| CallError::NotCallable)
        .context(CallSnafu {
            value: called.clone(),
        })?;

    let args: Vec<_> = expr
        .args
        .iter()
        .map(|arg| super::eval(arg, cx))
        .try_collect()?;

    cx.jail(|cx| {
        callable
            .call(cx.inject(), &args)
            .context(CallSnafu { value: called })
    })
}

pub(crate) fn var_use(call_expr: &CallExpr) -> crate::var_use::VarUse {
    VarUse::sequence(
        once(&call_expr.called)
            .chain(call_expr.args.iter())
            .map(super::var_use),
    )
}
