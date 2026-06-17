use dices_ast::expr::{
    Expr,
    unary::{UnOp, UnaryExpr},
};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, expr::build_expr};

pub(super) fn build_unary(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    if first.as_rule() == Rule::repeat {
        return build_expr(first, input);
    }
    let op = match first.as_str() {
        "+" => UnOp::Plus,
        "-" => UnOp::Minus,
        "!" => UnOp::Not,
        _ => unreachable!(),
    };
    let operand = build_expr(inner.next().unwrap(), input)?;
    Ok(Expr::Unary(Box::new(UnaryExpr { op, operand })))
}

pub(super) fn build_dice_unary(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    if first.as_rule() == Rule::dice_binary {
        return build_expr(first, input);
    }
    let operand = build_expr(inner.next().unwrap(), input)?;
    Ok(Expr::Unary(Box::new(UnaryExpr {
        op: UnOp::Dice,
        operand,
    })))
}