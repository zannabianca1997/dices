use dices_ast::expr::list::ListExpr;
use dices_values::{Value, list::ValueList};

use crate::{EvalError, context::Context};

pub(super) fn eval(expr: &ListExpr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let items: Result<Vec<_>, _> = expr.items.iter().map(|e| super::eval(e, cx)).collect();
    Ok(Value::List(ValueList::new(items?)))
}
