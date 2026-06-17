// Operator stubs intentionally accept their operand and context without using
// them yet; the real implementations land in a follow-up.
#![allow(unused_variables)]

use dices_ast::expr::unary::{UnOp, UnaryExpr};
use dices_values::Value;

use crate::{EvalError, context::Context};

pub fn eval(expr: &UnaryExpr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let operand = super::eval(&expr.operand, cx)?;
    match expr.op {
        // Math
        UnOp::Plus => eval_plus(operand, cx),
        UnOp::Minus => eval_minus(operand, cx),
        // Logic
        UnOp::Not => eval_not(operand, cx),
        // Misc
        UnOp::Dice => eval_dice(operand, cx),
    }
}

fn eval_plus(operand: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_minus(operand: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_not(operand: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_dice(operand: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}
