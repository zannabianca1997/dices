use std::{collections::BTreeMap, mem};

use dices_values::{
    Value, cast::push_down_if_injected, list::ValueList, map::ValueMap, string::ValueString,
};
use itertools::Itertools;

use crate::{EvalError, context::Context};

/// Push down all injected values
fn push_down_all_injected(values: &mut [Value]) -> Result<(), EvalError> {
    for dest in values {
        let value = mem::take(dest);
        *dest = push_down_if_injected(value)?;
    }
    Ok(())
}

pub fn join_all(values: &mut [Value], _cx: &mut Context<'_>) -> Result<Value, EvalError> {
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
