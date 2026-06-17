// Operator stubs intentionally accept their operands and context without using
// them yet; the real implementations land in a follow-up.
#![allow(unused_variables)]

use dices_ast::expr::binary::{BinOp, BinaryExpr};
use dices_values::Value;

use crate::{EvalError, context::Context, utils::join_all};

pub fn eval(expr: &BinaryExpr, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    let lhs = super::eval(&expr.lhs, cx)?;
    let rhs = super::eval(&expr.rhs, cx)?;
    match expr.op {
        // Math
        BinOp::Add => eval_add(lhs, rhs, cx),
        BinOp::Sub => eval_sub(lhs, rhs, cx),
        BinOp::Mul => eval_mul(lhs, rhs, cx),
        BinOp::Div => eval_div(lhs, rhs, cx),
        BinOp::Rem => eval_rem(lhs, rhs, cx),
        // Comparison
        BinOp::Eq => eval_eq(lhs, rhs, cx),
        BinOp::Ne => eval_ne(lhs, rhs, cx),
        BinOp::Lt => eval_lt(lhs, rhs, cx),
        BinOp::Gt => eval_gt(lhs, rhs, cx),
        BinOp::Le => eval_le(lhs, rhs, cx),
        BinOp::Ge => eval_ge(lhs, rhs, cx),
        // Logic
        BinOp::And => eval_and(lhs, rhs, cx),
        BinOp::Or => eval_or(lhs, rhs, cx),
        // Misc
        BinOp::Join => eval_join(lhs, rhs, cx),
        BinOp::Dice => eval_dice(lhs, rhs, cx),
        BinOp::Repeat => eval_repeat(lhs, rhs, cx),
    }
}

fn eval_add(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_sub(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_mul(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_div(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_rem(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_eq(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_ne(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_lt(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_gt(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_le(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_ge(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_and(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_or(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_join(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    join_all(&mut [lhs, rhs], cx)
}

fn eval_dice(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}

fn eval_repeat(lhs: Value, rhs: Value, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    todo!()
}
