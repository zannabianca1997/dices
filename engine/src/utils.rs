use std::{cmp::Ordering, collections::BTreeMap, mem};

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
            Value::Map(values) => deep_sum(values.values().cloned()),
            other => Ok(ValueInt::try_from(other)?),
        })
        .try_fold(ValueInt::ZERO, |a, b| b.map(|b| a + b))
}

/// Force the value to integer and apply a fallible function. If the value is a
/// collection, mantain it's shape and do it on it's elements instead.
pub fn deep_apply(value: Value, op: &mut impl FnMut(ValueInt) -> Result<ValueInt, EvalError>) -> Result<Value, EvalError> {
    match push_down_if_injected(value)? {
        Value::List(values) => values.into_iter().map(|value| deep_apply(value, op)).try_collect().map(Value::List),
        Value::Map(values) => values.into_iter().map(|(key, value)| deep_apply(value, op).map(|value| (key, value))).try_collect().map(Value::Map),
        other => op(ValueInt::try_from(other)?).map(Value::Int),
    }
}

/// Wrapper around a value that implements PartialOrd and Eq how the console should see it
///
/// [`Value`] has an implementation of `Ord`, but ordering between injected is
/// compiler-dependant. This fixes that, and also makes incomparable different classes of values
#[derive(Debug)]
#[repr(transparent)]
pub struct DicesOrd(pub Value);

impl DicesOrd {
    fn from_ref(value: &Value) -> &Self {
        unsafe {
            // Safety: `#[repr(transparent)]`
            &*(value as *const _ as *const _)
        }
    }

    fn from_slice_ref(slice: &[Value]) -> &[Self] {
        unsafe {
            // Safety: `#[repr(transparent)]`
            &*(slice as *const _ as *const _)
        }
    }
}


impl PartialEq for DicesOrd {
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl Eq for DicesOrd {}

impl PartialOrd for DicesOrd {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (&self.0, &other.0) {
            // scalar classes can be compared with themselves, except injected
            (Value::Null(a), Value::Null(b)) => Some(a.cmp(b)),
            (Value::Bool(a), Value::Bool(b)) => Some(a.cmp(b)),
            (Value::Int(a), Value::Int(b)) => Some(a.cmp(b)),
            (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
            // Lists compared lexicographically with this order
            (Value::List(a), Value::List(b)) => {
                Self::from_slice_ref(a.as_slice()).partial_cmp(Self::from_slice_ref(b.as_slice()))
            }
            // Maps: compare lexicographically the tuples of items sorted by
            // key, with the values wrapped in itself.
            (Value::Map(a), Value::Map(b)) => {
                // Ensure the backing map is still a btreemap. If this fail, the
                // iterators must be ordered with respect of the keys.
                let a: &BTreeMap<ValueString, _> = a;
                let b: &BTreeMap<ValueString, _> = b;

                let a_items = a.iter().map(|(k, v)| (k, Self::from_ref(v)));
                let b_items = b.iter().map(|(k, v)| (k, Self::from_ref(v)));

                Iterator::partial_cmp(a_items, b_items)
            }
            // Injected have only equality
            (Value::Injected(a), Value::Injected(b)) => (a == b).then_some(Ordering::Equal),
            // Bools can be compared with ints
            (Value::Bool(a), Value::Int(b)) => Some(ValueInt::from(*a).cmp(b)),
            (Value::Int(a), Value::Bool(b)) => Some(a.cmp(&ValueInt::from(*b))),

            // Every other comparison is unsupported
            _ => None,
        }
    }
}
