use dices_ast::expr::{
    Expr,
    binary::{BinOp, BinaryExpr},
};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, expr::build_expr};

pub(super) fn cmp_op_to_binop(s: &str) -> BinOp {
    match s {
        "==" => BinOp::Eq,
        "!=" => BinOp::Ne,
        "<" => BinOp::Lt,
        ">" => BinOp::Gt,
        "<=" => BinOp::Le,
        ">=" => BinOp::Ge,
        _ => unreachable!(),
    }
}

pub(super) fn add_op_to_binop(s: &str) -> BinOp {
    match s {
        "+" => BinOp::Add,
        "-" => BinOp::Sub,
        _ => unreachable!(),
    }
}

pub(super) fn mul_op_to_binop(s: &str) -> BinOp {
    match s {
        "*" => BinOp::Mul,
        "/" => BinOp::Div,
        "%" => BinOp::Rem,
        _ => unreachable!(),
    }
}

pub(super) fn build_binary_chain(
    pair: Pair<Rule>,
    op: BinOp,
    input: &ValueString,
) -> Result<Expr, ParseError> {
    let pairs: Vec<_> = pair.into_inner().collect();
    let mut lhs = build_expr(pairs[0].clone(), input)?;
    for rhs_pair in &pairs[1..] {
        let rhs = build_expr(rhs_pair.clone(), input)?;
        lhs = Expr::Binary(Box::new(BinaryExpr { lhs, op, rhs }));
    }
    Ok(lhs)
}

pub(super) fn build_operator_chain(
    pair: Pair<Rule>,
    op_fn: fn(&str) -> BinOp,
    input: &ValueString,
) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let mut lhs = build_expr(inner.next().unwrap(), input)?;
    loop {
        let Some(op_pair) = inner.next() else {
            break;
        };
        let op = op_fn(op_pair.as_str());
        let rhs_pair = inner.next().unwrap();
        let rhs = build_expr(rhs_pair, input)?;
        lhs = Expr::Binary(Box::new(BinaryExpr { lhs, op, rhs }));
    }
    Ok(lhs)
}