use std::{collections::BTreeMap, mem, ops::Add};

use dices_values::{
    Value, cast::push_down_if_injected, int::ValueInt, list::ValueList, map::ValueMap,
    string::ValueString,
};
use itertools::Itertools;
use num::traits::ConstZero;

use crate::EvalError;

/// Push down all injected values
fn push_down_all_injected(values: &mut [Value]) -> Result<(), EvalError> {
    for dest in values {
        let value = mem::take(dest);
        *dest = push_down_if_injected(value)?;
    }
    Ok(())
}

pub fn join_all(values: &mut [Value]) -> Result<Value, EvalError> {
    push_down_all_injected(values)?;

    // Map merge if all map
    if values.iter().all(|v| v.is_map()) {
        let res = values
            .iter_mut()
            .map(|v| mem::take(v).unwrap_map())
            .tree_reduce(ValueMap::join)
            .unwrap_or_default();
        return Ok(res.into());
    }

    // If there is at least a string, and no list is present, use string concatenation
    if values.iter().any(Value::is_string) && !values.iter().any(Value::is_list) {
        let res = values
            .iter_mut()
            .map(|v| {
                ValueString::try_from(mem::take(v)).expect("Injected should have been cast away")
            })
            .tree_reduce(ValueString::concat)
            .unwrap_or_default();
        return Ok(res.into());
    }

    // Default to list concatenation-creation
    let res = values
        .iter_mut()
        .map(|v| ValueList::try_from(mem::take(v)).expect("Injected should have been cast away"))
        .tree_reduce(ValueList::concat)
        .unwrap_or_default();
    return Ok(res.into());
}

/// Sum all the values, recursing inside containers
///
/// Used to implement sums and subs so that `3d6 + 3` works
pub fn deep_sum(values: impl IntoIterator<Item = Value>) -> Result<ValueInt, EvalError> {
    values
        .into_iter()
        .map(|value| match push_down_if_injected(value)? {
            Value::List(values) => deep_sum(values),
            Value::Map(value) => deep_sum(value.values().cloned()),
            other => Ok(ValueInt::try_from(other)?),
        })
        .try_fold(ValueInt::ZERO, |a, b| b.map(|b| a + b))
}
