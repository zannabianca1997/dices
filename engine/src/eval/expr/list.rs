use dices_ast::expr::list::ListExpr;
use dices_values::{Value, list::ValueList};

use crate::{EvalError, context::Context, var_use::VarUse};

pub(super) fn eval(expr: &ListExpr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    let items: Result<Vec<_>, _> = expr.items.iter().map(|e| super::eval(e, cx)).collect();
    Ok(Value::List(ValueList::new(items?)))
}

pub(crate) fn var_use(expr: &ListExpr) -> VarUse {
    VarUse::sequence(expr.items.iter().map(super::var_use))
}
