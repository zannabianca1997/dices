use dices_ast::expr::member_access::MemberAccessExpr;
use dices_values::{
    Value, cast::push_down_if_injected, int::ValueInt, list::ValueList, map::ValueMap,
    string::ValueString,
};
use num::{FromPrimitive, Integer, ToPrimitive, Zero, traits::ConstZero};

use crate::{EvalError, context::Context, var_use::VarUse};

pub(crate) fn eval(
    member_access_expr: &MemberAccessExpr,
    cx: &mut (impl Context + ?Sized),
) -> Result<Value, EvalError> {
    let container =
        push_down_if_injected(crate::eval::expr::eval(&member_access_expr.container, cx)?)?;

    let index = push_down_if_injected(crate::eval::expr::eval(&member_access_expr.index, cx)?)?;

    read_member(container, index)
}

/// Read a member out of a (already push-down'd) container value.
///
/// Shared between the expression evaluator and the assign path.
pub(crate) fn read_member(container: Value, index: Value) -> Result<Value, EvalError> {
    if !(container.is_map() || container.is_string() || container.is_list()) {
        return Err(EvalError::NonIndexable { container });
    }

    match (container, index) {
        (Value::Injected(_), _) | (_, Value::Injected(_)) => {
            unreachable!("push_down_if_injected would fail before returning those")
        }
        (Value::Null(_) | Value::Bool(_) | Value::Int(_), _) => unreachable!(),
        (Value::String(value_string), index) => index_string(value_string, index),
        (Value::List(value_list), index) => index_list(value_list, index),
        (Value::Map(value_map), index) => index_map(value_map, index),
    }
}

fn index_string(value_string: ValueString, index: Value) -> Result<Value, EvalError> {
    let len = ValueInt::from_usize(value_string.chars().count()).unwrap();
    if len.is_zero() {
        return Ok(ValueString::empty().into());
    }
    match into_sequence_index(index)? {
        SequenceIndex::Item { idx } => {
            let idx = idx.mod_floor(&len).to_usize().unwrap();

            let (byte_idx, ch) = value_string.char_indices().nth(idx).unwrap();

            Ok(value_string
                .slice(byte_idx..(byte_idx + ch.len_utf8()))
                .unwrap()
                .into())
        }
        SequenceIndex::Range { start, stop } => {
            let start = start
                .unwrap_or(ValueInt::ZERO)
                .mod_floor(&len)
                .to_usize()
                .unwrap();
            let stop = stop
                .map(|s| s.mod_floor(&len).to_usize().unwrap())
                .unwrap_or(value_string.chars().count());

            if start >= stop {
                return Ok(ValueString::empty().into());
            }

            let start_idx = value_string.char_indices().nth(start).unwrap().0;
            let stop_idx = if stop == value_string.chars().count() {
                value_string.len()
            } else {
                value_string.char_indices().nth(stop).unwrap().0
            };

            Ok(value_string.slice(start_idx..stop_idx).unwrap().into())
        }
    }
}

fn index_list(value_list: ValueList, index: Value) -> Result<Value, EvalError> {
    let len = ValueInt::from_usize(value_list.len()).unwrap();
    if len.is_zero() {
        return Ok(ValueList::empty().into());
    }
    match into_sequence_index(index)? {
        SequenceIndex::Item { idx } => {
            let idx = idx.mod_floor(&len).to_usize().unwrap();
            Ok(value_list[idx].clone())
        }
        SequenceIndex::Range { start, stop } => {
            let start = start
                .unwrap_or(ValueInt::ZERO)
                .mod_floor(&len)
                .to_usize()
                .unwrap();
            let stop = stop
                .map(|s| s.mod_floor(&len).to_usize().unwrap())
                .unwrap_or(value_list.len());

            if start >= stop {
                return Ok(ValueList::empty().into());
            }

            Ok(value_list.slice(start..stop).unwrap().into())
        }
    }
}

fn index_map(value_map: ValueMap, index: Value) -> Result<Value, EvalError> {
    let index = ValueString::try_from(index)?;

    Ok(value_map.get(&index).cloned().unwrap_or_default())
}

pub(crate) enum SequenceIndex {
    Item {
        idx: ValueInt,
    },
    Range {
        start: Option<ValueInt>,
        stop: Option<ValueInt>,
    },
}

pub(crate) fn into_sequence_index(index: Value) -> Result<SequenceIndex, EvalError> {
    Ok(match index {
        Value::Map(value_map) => SequenceIndex::Range {
            start: value_map
                .get("start")
                .cloned()
                .map(ValueInt::try_from)
                .transpose()?,
            stop: value_map
                .get("end")
                .cloned()
                .map(ValueInt::try_from)
                .transpose()?,
        },
        Value::List(value_list) if value_list.len() != 1 => {
            let [start, end] = &*value_list else {
                return Err(EvalError::IndexingWithListNeedLenghtTwo {
                    len: value_list.len(),
                });
            };

            SequenceIndex::Range {
                start: Some(start.clone().try_into()?),
                stop: Some(end.clone().try_into()?),
            }
        }
        Value::Injected(_) => unreachable!(),
        other => SequenceIndex::Item {
            idx: other.try_into()?,
        },
    })
}

pub(crate) fn var_use(member_access_expr: &MemberAccessExpr) -> VarUse {
    crate::eval::expr::var_use(&member_access_expr.container)
        .then(crate::eval::expr::var_use(&member_access_expr.index))
}

/// Assign `rhs` to `index` inside `container`, returning a **new** container
/// value with the member updated.
///
/// `container` and `index` must already be push-down'd (no `Value::Injected`).
/// Shared with the assign path.
pub(crate) fn assign_member(
    container: Value,
    index: Value,
    rhs: Value,
) -> Result<Value, EvalError> {
    match container {
        Value::Injected(_)
        | Value::Null(_)
        | Value::Bool(_)
        | Value::Int(_) => Err(EvalError::NonIndexable { container }),
        Value::String(value_string) => assign_string_member(value_string, index, rhs),
        Value::List(value_list) => assign_list_member(value_list, index, rhs),
        Value::Map(value_map) => assign_map_member(value_map, index, rhs),
    }
}

fn assign_map_member(
    value_map: ValueMap,
    index: Value,
    rhs: Value,
) -> Result<Value, EvalError> {
    let key = ValueString::try_from(index)?;
    let mut inner: std::collections::BTreeMap<ValueString, Value> =
        std::collections::BTreeMap::clone(&value_map);
    inner.insert(key, rhs);
    Ok(ValueMap::new(inner).into())
}

fn assign_list_member(
    value_list: ValueList,
    index: Value,
    rhs: Value,
) -> Result<Value, EvalError> {
    let len = ValueInt::from_usize(value_list.len()).unwrap();
    if len.is_zero() {
        return Err(EvalError::NonIndexable {
            container: Value::List(value_list),
        });
    }
    match into_sequence_index(index)? {
        SequenceIndex::Item { idx } => {
            let idx = idx.mod_floor(&len).to_usize().unwrap();
            let mut vec: Vec<Value> = value_list.as_slice().to_vec();
            vec[idx] = rhs;
            Ok(ValueList::new(vec).into())
        }
        SequenceIndex::Range { start, stop } => {
            // Cast the rhs to a list: a non-list value becomes a single
            // element list, matching the `to_list` conversion semantics.
            let rhs_list = ValueList::try_from(rhs)?;
            let start = start
                .unwrap_or(ValueInt::ZERO)
                .mod_floor(&len)
                .to_usize()
                .unwrap();
            let stop = stop
                .map(|s| s.mod_floor(&len).to_usize().unwrap())
                .unwrap_or(value_list.len());

            let mut vec: Vec<Value> = value_list.as_slice().to_vec();
            let replacement: Vec<Value> = rhs_list.as_slice().to_vec();
            if start >= stop {
                // insert at `start`, removing nothing
                for (i, v) in replacement.into_iter().enumerate() {
                    vec.insert(start + i, v);
                }
            } else {
                vec.splice(start..stop, replacement);
            }
            Ok(ValueList::new(vec).into())
        }
    }
}

fn assign_string_member(
    value_string: ValueString,
    index: Value,
    rhs: Value,
) -> Result<Value, EvalError> {
    // Cast the rhs to a string, matching the `to_string` conversion
    // semantics. This lets `a[0] = 42` work, inserting "42" into the string.
    let rhs_string = ValueString::try_from(rhs)?;
    let len = ValueInt::from_usize(value_string.chars().count()).unwrap();
    if len.is_zero() {
        return Err(EvalError::NonIndexable {
            container: Value::String(value_string),
        });
    }
    match into_sequence_index(index)? {
        SequenceIndex::Item { idx } => {
            let idx = idx.mod_floor(&len).to_usize().unwrap();
            let (byte_idx, ch) = value_string.char_indices().nth(idx).unwrap();
            let mut result = String::with_capacity(
                value_string.len() - ch.len_utf8() + rhs_string.len(),
            );
            result.push_str(&value_string.as_str()[..byte_idx]);
            result.push_str(rhs_string.as_str());
            result.push_str(&value_string.as_str()[byte_idx + ch.len_utf8()..]);
            Ok(ValueString::new(result).into())
        }
        SequenceIndex::Range { start, stop } => {
            let char_count = value_string.chars().count();
            let start = start
                .unwrap_or(ValueInt::ZERO)
                .mod_floor(&len)
                .to_usize()
                .unwrap();
            let stop = stop
                .map(|s| s.mod_floor(&len).to_usize().unwrap())
                .unwrap_or(char_count);

            let start_byte = value_string.char_indices().nth(start).unwrap().0;
            let stop_byte = if start >= stop {
                // insert at `start`, removing nothing
                start_byte
            } else if stop == char_count {
                value_string.len()
            } else {
                value_string.char_indices().nth(stop).unwrap().0
            };

            let mut result = String::with_capacity(
                value_string.len() - (stop_byte - start_byte) + rhs_string.len(),
            );
            result.push_str(&value_string.as_str()[..start_byte]);
            result.push_str(rhs_string.as_str());
            result.push_str(&value_string.as_str()[stop_byte..]);
            Ok(ValueString::new(result).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use dices_values::{
        Value,
        int::ValueInt,
        list::ValueList,
        map::ValueMap,
        string::ValueString,
    };
    use std::collections::BTreeMap;

    use super::{assign_member, read_member};
    use crate::EvalError;
    use num::FromPrimitive;

    fn s(v: &'static str) -> Value {
        ValueString::new_static(v).into()
    }
    fn i(n: i64) -> Value {
        ValueInt::from_i64(n).unwrap().into()
    }
    fn list(vs: Vec<Value>) -> Value {
        ValueList::new(vs).into()
    }
    fn key(k: &'static str) -> ValueString {
        ValueString::new_static(k)
    }

    #[test]
    fn string_range_replace() {
        // "Hello"[[1..4]] = "er"  ->  "Hero"
        let container = s("Hello");
        let index = list(vec![i(1), i(4)]);
        let new = assign_member(container, index, s("er")).unwrap();
        assert_eq!(new, s("Hero"));
    }

    #[test]
    fn string_item_replace() {
        // "Hello"[0] = "J"  ->  "Jello"
        let new = assign_member(s("Hello"), i(0), s("J")).unwrap();
        assert_eq!(new, s("Jello"));
    }

    #[test]
    fn string_item_negative_wraps() {
        // "Hello"[-1] = "y": -1 % 5 = 4, so last char 'o' -> 'y'  ->  "Helly"
        let new = assign_member(s("Hello"), i(-1), s("y")).unwrap();
        assert_eq!(new, s("Helly"));
    }

    #[test]
    fn string_insert_at_empty_range() {
        // "Hello"[[2..2]] = "XY"  ->  "HeXYllo" (insert, no removal)
        let new = assign_member(s("Hello"), list(vec![i(2), i(2)]), s("XY")).unwrap();
        assert_eq!(new, s("HeXYllo"));
    }

    #[test]
    fn string_assign_non_string_rhs_is_cast_to_string() {
        // `a[0] = 42` casts 42 to "42" via to_string semantics.
        let new = assign_member(s("Hello"), i(0), i(42)).unwrap();
        assert_eq!(new, s("42ello"));
    }

    #[test]
    fn empty_string_item_errors() {
        let err = assign_member(s(""), i(0), s("x")).unwrap_err();
        assert!(matches!(err, EvalError::NonIndexable { .. }));
    }

    #[test]
    fn list_item_replace() {
        let new = assign_member(list(vec![i(1), i(2), i(3)]), i(1), s("x")).unwrap();
        assert_eq!(new, list(vec![i(1), s("x"), i(3)]));
    }

    #[test]
    fn list_range_splice() {
        // [1,2,3,4,5][[1..3]] = [9,9,9]  ->  [1,9,9,9,4,5]
        let new = assign_member(
            list(vec![i(1), i(2), i(3), i(4), i(5)]),
            list(vec![i(1), i(3)]),
            list(vec![i(9), i(9), i(9)]),
        )
        .unwrap();
        assert_eq!(new, list(vec![i(1), i(9), i(9), i(9), i(4), i(5)]));
    }

    #[test]
    fn list_insert_at_empty_range() {
        // [1,2,3][[1..1]] = [9]  ->  [1,9,2,3]
        let new = assign_member(
            list(vec![i(1), i(2), i(3)]),
            list(vec![i(1), i(1)]),
            list(vec![i(9)]),
        )
        .unwrap();
        assert_eq!(new, list(vec![i(1), i(9), i(2), i(3)]));
    }

    #[test]
    fn list_range_non_list_rhs_is_wrapped_into_single_element_list() {
        // `a[[0..1]] = 9` wraps 9 into [9] via to_list semantics.
        let new = assign_member(list(vec![i(1), i(2)]), list(vec![i(0), i(1)]), i(9)).unwrap();
        assert_eq!(new, list(vec![i(9), i(2)]));
    }

    #[test]
    fn empty_list_item_errors() {
        let err = assign_member(list(vec![]), i(0), i(1)).unwrap_err();
        assert!(matches!(err, EvalError::NonIndexable { .. }));
    }

    #[test]
    fn map_insert_new_key() {
        let mut map = BTreeMap::new();
        map.insert(key("a"), i(1));
        let container = ValueMap::new(map).into();
        let new = assign_member(container, s("b"), i(2)).unwrap();
        // Read it back
        let read = read_member(new, s("b")).unwrap();
        assert_eq!(read, i(2));
    }

    #[test]
    fn map_overwrite_key() {
        let mut map = BTreeMap::new();
        map.insert(key("a"), i(1));
        let container = ValueMap::new(map).into();
        let new = assign_member(container, s("a"), s("x")).unwrap();
        let read = read_member(new, s("a")).unwrap();
        assert_eq!(read, s("x"));
    }

    #[test]
    fn non_indexable_container_errors() {
        let err = assign_member(i(42), i(0), i(1)).unwrap_err();
        assert!(matches!(err, EvalError::NonIndexable { .. }));
    }

    #[test]
    fn read_after_assign_roundtrip() {
        // Assign then read should give back the assigned value
        let new = assign_member(s("Hello"), i(0), s("J")).unwrap();
        assert_eq!(read_member(new, i(0)).unwrap(), s("J"));
    }
}
