//! Unary operator implementation

use dices_ast::expr::unary::{UnOp, UnaryExpr};
use dices_values::{Value, cast::push_down_if_injected, int::ValueInt, utils::deep_sum};

use crate::{EvalError, context::Context, var_use::VarUse};

pub fn eval(expr: &UnaryExpr, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    let operand = push_down_if_injected(super::eval(&expr.operand, cx)?)?;
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

pub fn var_use(expr: &UnaryExpr) -> VarUse {
    super::var_use(&expr.operand)
}

fn eval_plus(operand: Value) -> Result<Value, EvalError> {
    deep_sum([operand]).map(Value::from).map_err(Into::into)
}

fn eval_minus(operand: Value) -> Result<Value, EvalError> {
    deep_sum([operand])
        .map(|int| Value::from(-int))
        .map_err(Into::into)
}

fn eval_not(operand: Value) -> Result<Value, EvalError> {
    Ok(Value::Bool(!operand.try_into()?))
}

fn eval_dice(operand: Value, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    let faces = ValueInt::try_from(operand)?;

    Ok(cx.dice(faces).into())
}
