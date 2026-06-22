// Operator stubs intentionally accept their operands and context without using
// them yet; the real implementations land in a follow-up.
#![allow(unused_variables)]

use std::cmp::Ordering;

use dices_ast::expr::{
    Expr,
    binary::{BinOp, BinaryExpr},
    unary::{UnOp, UnaryExpr},
};
use dices_values::{
    Value, bool::ValueBool, cast::push_down_if_injected, int::ValueInt, list::ValueList,
    map::ValueMap,
};
use num::{Integer, ToPrimitive, Zero, traits::ConstZero};
use snafu::OptionExt;

use crate::{
    EvalError, IncomparableValuesSnafu,
    context::Context,
    utils::{DicesOrd, deep_apply, deep_sum, join_all},
    var_use::VarUse,
};

pub fn eval(expr: &BinaryExpr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    // Repeat does not evaluate the body
    if expr.op == BinOp::Repeat {
        let rhs = super::eval(&expr.rhs, cx)?;
        return eval_repeat(&expr.lhs, rhs, cx);
    }

    // And and or are short circuiting
    if expr.op == BinOp::And || expr.op == BinOp::Or {
        let lhs = super::eval(&expr.lhs, cx)?;
        return match expr.op {
            // Logic
            BinOp::And => eval_and(lhs, &expr.rhs, cx),
            BinOp::Or => eval_or(lhs, &expr.rhs, cx),

            _ => unreachable!(),
        };
    }

    let lhs = push_down_if_injected(super::eval(&expr.lhs, cx)?)?;
    let rhs = push_down_if_injected(super::eval(&expr.rhs, cx)?)?;
    match expr.op {
        // Math
        BinOp::Add => eval_add(lhs, rhs),
        BinOp::Sub => eval_sub(lhs, rhs),
        BinOp::Mul => eval_mul(lhs, rhs),
        BinOp::Div => eval_div(lhs, rhs),
        BinOp::Rem => eval_rem(lhs, rhs),
        // Comparison
        BinOp::Eq => eval_eq(lhs, rhs),
        BinOp::Ne => eval_ne(lhs, rhs),
        BinOp::Lt => eval_lt(lhs, rhs),
        BinOp::Gt => eval_gt(lhs, rhs),
        BinOp::Le => eval_le(lhs, rhs),
        BinOp::Ge => eval_ge(lhs, rhs),
        // Misc
        BinOp::Join => eval_join(lhs, rhs),
        BinOp::Dice => eval_dice(lhs, rhs, cx),
        // Filters
        BinOp::KeepHigh => eval_filter(lhs, rhs, FilterKind::KeepHigh),
        BinOp::KeepLow => eval_filter(lhs, rhs, FilterKind::KeepLow),
        BinOp::RemoveHigh => eval_filter(lhs, rhs, FilterKind::RemoveHigh),
        BinOp::RemoveLow => eval_filter(lhs, rhs, FilterKind::RemoveLow),
        // Handled differently
        BinOp::Repeat | BinOp::And | BinOp::Or => unreachable!(),
    }
}

pub fn var_use(expr: &BinaryExpr) -> VarUse {
    let lhs = super::var_use(&expr.lhs);
    let rhs = super::var_use(&expr.rhs);

    // Repeat evaluate the index first.
    //
    // The body repeats, but `then` is idempotent. This might be a little
    // conservative as in the repeat can be 0, but that is not provable now.
    // Plus that would give problems if a variable _might_ be set, but that's
    // not possible as the body is an expression.
    if expr.op == BinOp::Repeat {
        debug_assert!(lhs.lets.is_empty(), "Expressions cannot define variables");

        return rhs.then(lhs);
    }

    // And and or are short circuiting, so they can skip the rhs. But that's not
    // important for the same reason that is not important for the repeat body.
    if expr.op == BinOp::And || expr.op == BinOp::Or {
        debug_assert!(rhs.lets.is_empty(), "Expressions cannot define variables");
    }

    return lhs.then(rhs);
}

fn eval_add(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    deep_sum([lhs, rhs]).map(Value::from)
}

fn eval_sub(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(Value::from(deep_sum([lhs])? - deep_sum([rhs])?))
}

fn eval_mul(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    match ValueInt::try_from(lhs.clone()) {
        Ok(lhs) => deep_apply(rhs, &mut |value| Ok(lhs.clone() * value)),
        Err(lhs_err) => match ValueInt::try_from(rhs) {
            Ok(rhs) => deep_apply(lhs, &mut |value| Ok(value * rhs.clone())),
            Err(rhs_err) => Err(EvalError::MulBetweenNonScalars {
                lhs: lhs_err,
                rhs: rhs_err,
            }),
        },
    }
}

fn eval_div(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    let rhs = ValueInt::try_from(rhs)?;
    if rhs.is_zero() {
        return Err(EvalError::DivisionByZero);
    }
    deep_apply(lhs, &mut |value| Ok(value / rhs.clone()))
}

fn eval_rem(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    let rhs = ValueInt::try_from(rhs)?;
    if rhs.is_zero() {
        return Err(EvalError::DivisionByZero);
    }
    deep_apply(lhs, &mut |value| Ok(value % rhs.clone()))
}

fn eval_eq(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(DicesOrd(lhs) == DicesOrd(rhs)).into())
}

fn eval_ne(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(DicesOrd(lhs) != DicesOrd(rhs)).into())
}

fn eval_lt(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(
        DicesOrd::from_ref(&lhs)
            .partial_cmp(DicesOrd::from_ref(&rhs))
            .context(IncomparableValuesSnafu { lhs, rhs })?
            == Ordering::Less,
    )
    .into())
}

fn eval_gt(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(
        DicesOrd::from_ref(&lhs)
            .partial_cmp(DicesOrd::from_ref(&rhs))
            .context(IncomparableValuesSnafu { lhs, rhs })?
            == Ordering::Greater,
    )
    .into())
}

fn eval_le(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(
        DicesOrd::from_ref(&lhs)
            .partial_cmp(DicesOrd::from_ref(&rhs))
            .context(IncomparableValuesSnafu { lhs, rhs })?
            != Ordering::Greater,
    )
    .into())
}

fn eval_ge(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(
        DicesOrd::from_ref(&lhs)
            .partial_cmp(DicesOrd::from_ref(&rhs))
            .context(IncomparableValuesSnafu { lhs, rhs })?
            != Ordering::Less,
    )
    .into())
}

fn eval_and(lhs: Value, rhs: &Expr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    let lhs_truthy = ValueBool::try_from(lhs.clone())?;

    if !lhs_truthy.get() {
        return Ok(lhs);
    }

    super::eval(rhs, cx)
}

fn eval_or(lhs: Value, rhs: &Expr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    let lhs_truthy = ValueBool::try_from(lhs.clone())?;

    if lhs_truthy.get() {
        return Ok(lhs);
    }

    super::eval(rhs, cx)
}

fn eval_join(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    join_all(&mut [lhs, rhs])
}

fn eval_dice(lhs: Value, rhs: Value, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    // desugar XdY -> (dY) ^ X
    eval_repeat(
        &Expr::Unary(Box::new(UnaryExpr {
            op: UnOp::Dice,
            operand: Expr::Const(Box::new(rhs)),
        })),
        lhs,
        cx,
    )
}

fn eval_repeat(
    lhs: &Expr,
    rhs: Value,
    cx: &mut (impl Context + ?Sized),
) -> Result<Value, EvalError> {
    let mut times = ValueInt::try_from(rhs)?.max(ValueInt::ZERO);
    let mut values = Vec::with_capacity(times.to_usize().unwrap_or(usize::MAX));
    while times > ValueInt::ZERO {
        values.push(super::eval(lhs, cx)?);
        times.dec();
    }
    Ok(Value::List(ValueList::new(values)))
}

enum FilterKind {
    KeepHigh,
    KeepLow,
    RemoveHigh,
    RemoveLow,
}

fn eval_filter(mut collection: Value, rhs: Value, kind: FilterKind) -> Result<Value, EvalError> {
    if !(collection.is_list() || collection.is_map()) {
        collection = ValueList::try_from(collection)?.into()
    }

    let count = ValueInt::try_from(rhs)?
        .max(ValueInt::ZERO)
        .to_usize()
        .unwrap_or(usize::MAX);

    match collection {
        Value::List(list) => {
            let len = list.len();
            let count = count.min(len);
            let mut indexed: Vec<(usize, Value)> = list.into_iter().enumerate().collect();
            indexed.sort_by(|(i1, v1), (i2, v2)| {
                DicesOrd(v1.clone())
                    .partial_cmp(&DicesOrd(v2.clone()))
                    .unwrap_or_else(|| i1.cmp(i2))
            });
            let selected: Vec<_> = match kind {
                FilterKind::KeepHigh => indexed.into_iter().skip(len - count).collect(),
                FilterKind::KeepLow => indexed.into_iter().take(count).collect(),
                FilterKind::RemoveHigh => indexed.into_iter().take(len - count).collect(),
                FilterKind::RemoveLow => indexed.into_iter().skip(count).collect(),
            };
            let mut selected = selected;
            selected.sort_by_key(|(i, _)| *i);
            Ok(Value::List(ValueList::from_iter(
                selected.into_iter().map(|(_, v)| v),
            )))
        }
        Value::Map(map) => {
            let len = map.len();
            let count = count.min(len);
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|(_, v1), (_, v2)| {
                DicesOrd(v1.clone())
                    .partial_cmp(&DicesOrd(v2.clone()))
                    .unwrap_or(Ordering::Equal)
            });
            let selected: std::collections::BTreeMap<_, _> = match kind {
                FilterKind::KeepHigh => entries.into_iter().rev().take(count).collect(),
                FilterKind::KeepLow => entries.into_iter().take(count).collect(),
                FilterKind::RemoveHigh => entries
                    .into_iter()
                    .take(len.saturating_sub(count))
                    .collect(),
                FilterKind::RemoveLow => entries
                    .into_iter()
                    .rev()
                    .take(len.saturating_sub(count))
                    .collect(),
            };
            Ok(Value::Map(ValueMap::new(selected)))
        }
        _ => unreachable!(),
    }
}
