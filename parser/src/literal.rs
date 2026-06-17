use std::str::FromStr;

use dices_ast::{expr::Expr, literal::Literal};
use dices_values::{
    bool::ValueBool,
    int::ValueInt,
    null::ValueNull,
    string::ValueString,
};
use pest::iterators::Pair;
use snafu::ResultExt;

use crate::{ParseError, Rule, expr::build_expr};

pub(super) fn build_literal(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Expr, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    build_expr(inner, input)
}

pub(super) fn build_int(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let s = pair.as_str();
    let i = ValueInt::from_str(s).context(crate::IntParseSnafu)?;
    Ok(Expr::Literal(Box::new(Literal::Int(i))))
}

pub(super) fn build_string(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Expr, ParseError> {
    let span = pair.as_span();
    let range = (span.start() + 1)..(span.end() - 1);
    let s = input
        .slice(range)
        .unwrap()
        .unescape()
        .context(crate::StringUnescapeSnafu)?;
    Ok(Expr::Literal(Box::new(Literal::String(s))))
}

pub(super) fn build_bool(pair: Pair<Rule>) -> Result<Expr, ParseError> {
    let b = match pair.as_str() {
        "true" => ValueBool::TRUE,
        "false" => ValueBool::FALSE,
        _ => unreachable!(),
    };
    Ok(Expr::Literal(Box::new(Literal::Bool(b))))
}

pub(super) fn build_null(_pair: Pair<Rule>) -> Expr {
    Expr::Literal(Box::new(Literal::Null(ValueNull)))
}