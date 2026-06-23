use dices_ast::expr::member_access::MemberAccessExpr;
use dices_values::{
    Value, cast::push_down_if_injected, int::ValueInt, list::ValueList, map::ValueMap,
    string::ValueString,
};
use num::{FromPrimitive, Integer, ToPrimitive, traits::ConstZero};

use crate::{EvalError, context::Context, var_use::VarUse};

pub(crate) fn eval(
    member_access_expr: &MemberAccessExpr,
    cx: &mut (impl Context + ?Sized),
) -> Result<Value, EvalError> {
    let container =
        push_down_if_injected(crate::eval::expr::eval(&member_access_expr.container, cx)?)?;

    if !(container.is_map() || container.is_string() || container.is_list()) {
        return Err(EvalError::NonIndexable { container });
    }

    let index = push_down_if_injected(crate::eval::expr::eval(&member_access_expr.index, cx)?)?;

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
    let mut len = ValueInt::from_usize(value_string.chars().count()).unwrap();
    len.dec();
    match into_sequence_index(index)? {
        SequenceIndex::Item { idx } => {
            let idx = idx.clamp(ValueInt::ZERO, len).to_usize().unwrap();

            let (idx, ch) = value_string.char_indices().nth(idx).unwrap();

            Ok(value_string
                .slice(idx..(idx + ch.len_utf8()))
                .unwrap()
                .into())
        }
        SequenceIndex::Range { start, stop } => {
            let start = start
                .unwrap_or(ValueInt::ZERO)
                .clamp(ValueInt::ZERO, len.clone())
                .to_usize()
                .unwrap();
            let stop = stop
                .unwrap_or(len.clone())
                .clamp(ValueInt::ZERO, len)
                .to_usize()
                .unwrap();

            if start >= stop {
                return Ok(ValueString::empty().into());
            }

            let mut indices = value_string.char_indices();

            let (start_idx, _) = indices.nth(start).unwrap();
            let (stop_idx, _) = indices.nth(stop - start - 1).unwrap();

            Ok(value_string.slice(start_idx..stop_idx).unwrap().into())
        }
    }
}

fn index_list(value_list: ValueList, index: Value) -> Result<Value, EvalError> {
    let mut len = ValueInt::from_usize(value_list.len()).unwrap();
    len.dec();
    match into_sequence_index(index)? {
        SequenceIndex::Item { idx } => {
            let idx = idx.clamp(ValueInt::ZERO, len).to_usize().unwrap();
            Ok(value_list[idx].clone())
        }
        SequenceIndex::Range { start, stop } => {
            let start = start
                .unwrap_or(ValueInt::ZERO)
                .clamp(ValueInt::ZERO, len.clone())
                .to_usize()
                .unwrap();
            let stop = stop
                .unwrap_or(len.clone())
                .clamp(ValueInt::ZERO, len)
                .to_usize()
                .unwrap();

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

enum SequenceIndex {
    Item {
        idx: ValueInt,
    },
    Range {
        start: Option<ValueInt>,
        stop: Option<ValueInt>,
    },
}

fn into_sequence_index(index: Value) -> Result<SequenceIndex, EvalError> {
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
