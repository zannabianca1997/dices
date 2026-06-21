use std::collections::BTreeMap;

use dices_ast::expr::map::MapExpr;
use dices_values::{Value, map::ValueMap};

use crate::{EvalError, context::Context};

pub(super) fn eval(expr: &MapExpr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let mut map = BTreeMap::new();
    for (key, value_expr) in &expr.items {
        let value = super::eval(value_expr, cx)?;
        map.insert(key.0.clone(), value);
    }
    Ok(Value::Map(ValueMap::new(map)))
}
