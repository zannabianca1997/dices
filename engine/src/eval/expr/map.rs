use std::collections::BTreeMap;

use dices_ast::expr::map::MapExpr;
use dices_values::{Value, map::ValueMap};

use crate::{EvalError, context::Context, var_use::VarUse};

pub(super) fn eval(expr: &MapExpr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    let mut map = BTreeMap::new();
    for (key, value_expr) in &expr.items {
        let value = super::eval(value_expr, cx)?;
        map.insert(key.0.clone(), value);
    }
    Ok(Value::Map(ValueMap::new(map)))
}

pub(crate) fn var_use(expr: &MapExpr) -> VarUse {
    VarUse::sequence(expr.items.iter().map(|(_, v)| super::var_use(v)))
}
