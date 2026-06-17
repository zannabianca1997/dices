// Operator stubs intentionally accept their operand and context without using
// them yet; the real implementations land in a follow-up.
#![allow(unused_variables)]

use dices_ast::expr::unary::{UnOp, UnaryExpr};
use dices_values::{Value, int::ValueInt};

use crate::{EvalError, context::Context, utils::deep_sum};

pub fn eval(expr: &UnaryExpr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let operand = super::eval(&expr.operand, cx)?;
    match expr.op {
        // Math
        UnOp::Plus => eval_plus(operand),
        UnOp::Minus => eval_minus(operand),
        // Logic
        UnOp::Not => eval_not(operand),
        // Misc
        UnOp::Dice => eval_dice(operand, cx),
    }
}

fn eval_plus(operand: Value) -> Result<Value, EvalError> {
    deep_sum([operand]).map(Value::from)
}

fn eval_minus(operand: Value) -> Result<Value, EvalError> {
    deep_sum([operand]).map(|int| Value::from(-int))
}

fn eval_not(operand: Value) -> Result<Value, EvalError> {
    Ok(Value::Bool(!operand.try_into()?))
}

fn eval_dice(operand: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let faces = ValueInt::try_from(operand)?;

    Ok(cx.dice(faces).into())
}
