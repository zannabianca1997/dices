// Operator stubs intentionally accept their operands and context without using
// them yet; the real implementations land in a follow-up.
#![allow(unused_variables)]

use dices_ast::expr::{
    Expr,
    binary::{BinOp, BinaryExpr},
    unary::{UnOp, UnaryExpr},
};
use dices_values::{Value, bool::ValueBool, int::ValueInt, list::ValueList};
use num::{Integer, ToPrimitive, Zero, traits::ConstZero};

use crate::{
    EvalError,
    context::Context,
    utils::{DicesOrd, deep_apply, deep_sum, join_all},
};

pub fn eval(expr: &BinaryExpr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
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

    let lhs = super::eval(&expr.lhs, cx)?;
    let rhs = super::eval(&expr.rhs, cx)?;
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
        // Handled differently
        BinOp::Repeat | BinOp::And | BinOp::Or => unreachable!(),
    }
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
    Ok(ValueBool::from(DicesOrd(lhs) < DicesOrd(rhs)).into())
}

fn eval_gt(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(DicesOrd(lhs) > DicesOrd(rhs)).into())
}

fn eval_le(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(DicesOrd(lhs) <= DicesOrd(rhs)).into())
}

fn eval_ge(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    Ok(ValueBool::from(DicesOrd(lhs) >= DicesOrd(rhs)).into())
}

fn eval_and(lhs: Value, rhs: &Expr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let lhs_truthy = ValueBool::try_from(lhs.clone())?;

    if !lhs_truthy.get() {
        return Ok(lhs);
    }

    super::eval(rhs, cx)
}

fn eval_or(lhs: Value, rhs: &Expr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let lhs_truthy = ValueBool::try_from(lhs.clone())?;

    if lhs_truthy.get() {
        return Ok(lhs);
    }

    super::eval(rhs, cx)
}

fn eval_join(lhs: Value, rhs: Value) -> Result<Value, EvalError> {
    join_all(&mut [lhs, rhs])
}

fn eval_dice(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
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

fn eval_repeat(lhs: &Expr, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let mut times = ValueInt::try_from(rhs)?.max(ValueInt::ZERO);
    let mut values = Vec::with_capacity(times.to_usize().unwrap_or(usize::MAX));
    while times > ValueInt::ZERO {
        values.push(super::eval(lhs, cx)?);
        times.dec();
    }
    Ok(Value::List(ValueList::new(values)))
}
