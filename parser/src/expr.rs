use std::sync::LazyLock;

use dices_ast::{
    expr::{
        Expr,
        binary::{BinOp, BinaryExpr},
        unary::{UnOp, UnaryExpr},
    },
    literal::{Literal, LiteralBool, LiteralNull},
};
use dices_values::{bool::ValueBool, null::ValueNull, string::ValueString};
use pest::pratt_parser::{Assoc, Op, PrattParser};

use crate::{ParseCommandError, Rule, literal};

pub(crate) mod binary;
pub(crate) mod call;
pub(crate) mod closure;
pub(crate) mod list;
pub(crate) mod map;
pub(crate) mod member_access;
pub(crate) mod scope;
pub(crate) mod unary;

static PRATT: LazyLock<PrattParser<Rule>> = LazyLock::new(|| {
    use Rule::*;
    PrattParser::new()
        .op(Op::prefix(closure_prefix))
        .op(Op::infix(join_op, Assoc::Left))
        .op(Op::infix(or_op, Assoc::Left))
        .op(Op::infix(and_op, Assoc::Left))
        .op(Op::infix(eq, Assoc::Left)
            | Op::infix(ne, Assoc::Left)
            | Op::infix(lt, Assoc::Left)
            | Op::infix(le, Assoc::Left)
            | Op::infix(gt, Assoc::Left)
            | Op::infix(ge, Assoc::Left))
        .op(Op::infix(plus_infix, Assoc::Left) | Op::infix(minus_infix, Assoc::Left))
        .op(Op::infix(mul, Assoc::Left) | Op::infix(div, Assoc::Left) | Op::infix(rem, Assoc::Left))
        .op(Op::prefix(plus_prefix) | Op::prefix(minus_prefix) | Op::prefix(not_op))
        .op(Op::infix(kh, Assoc::Left)
            | Op::infix(kl, Assoc::Left)
            | Op::infix(rh, Assoc::Left)
            | Op::infix(rl, Assoc::Left))
        .op(Op::infix(repeat_op, Assoc::Left))
        .op(Op::prefix(dice_keyword))
        .op(Op::infix(dice_infix, Assoc::Left))
        .op(Op::postfix(call_args) | Op::postfix(member_access))
});

pub(crate) fn build_expr(
    pair: pest::iterators::Pair<Rule>,
    input: &ValueString,
) -> Result<Expr, ParseCommandError> {
    match pair.as_rule() {
        Rule::main => {
            let inner = pair.into_inner().next().unwrap();
            build_expr(inner, input)
        }
        Rule::expr => PRATT
            .map_primary(|primary| match primary.as_rule() {
                Rule::int => literal::build_int_literal(primary)
                    .map(|i| Expr::Literal(Box::new(Literal::Int(i)))),
                Rule::string => literal::build_string_value(primary, input)
                    .map(|s| Expr::Literal(Box::new(Literal::String(s)))),
                Rule::bool => {
                    let b = match primary.as_str() {
                        "true" => ValueBool::TRUE,
                        "false" => ValueBool::FALSE,
                        _ => unreachable!(),
                    };
                    Ok(Expr::Literal(Box::new(Literal::Bool(LiteralBool(b)))))
                }
                Rule::null => Ok(Expr::Literal(Box::new(Literal::Null(LiteralNull(
                    ValueNull,
                ))))),
                Rule::paren_expr => {
                    let inner = primary.into_inner().next().unwrap();
                    build_expr(inner, input)
                }
                Rule::scope => scope::build_scope_expr(primary, input),
                Rule::list => list::build_list_expr(primary, input),
                Rule::map => map::build_map_expr(primary, input),
                Rule::identifier => crate::identifier::parse_identifier(primary, input)
                    .map(|ident| Expr::Variable(Box::new(ident))),
                Rule::std => Ok(Expr::Std),
                r => crate::unexpected_rule(r),
            })
            .map_prefix(|op, rhs| {
                let op_rule = op.as_rule();
                match op_rule {
                    Rule::closure_prefix => closure::build_closure_expr(op, rhs?, input),
                    Rule::plus_prefix | Rule::minus_prefix | Rule::not_op | Rule::dice_keyword => {
                        let rhs = rhs?;
                        let op = match op_rule {
                            Rule::plus_prefix => UnOp::Plus,
                            Rule::minus_prefix => UnOp::Minus,
                            Rule::not_op => UnOp::Not,
                            Rule::dice_keyword => UnOp::Dice,
                            _ => unreachable!(),
                        };
                        Ok(Expr::Unary(Box::new(UnaryExpr { op, operand: rhs })))
                    }
                    _ => unreachable!(),
                }
            })
            .map_infix(|lhs, op, rhs| {
                let lhs = lhs?;
                let rhs = rhs?;
                let op = match op.as_rule() {
                    Rule::join_op => BinOp::Join,
                    Rule::or_op => BinOp::Or,
                    Rule::and_op => BinOp::And,
                    Rule::eq => BinOp::Eq,
                    Rule::ne => BinOp::Ne,
                    Rule::lt => BinOp::Lt,
                    Rule::le => BinOp::Le,
                    Rule::gt => BinOp::Gt,
                    Rule::ge => BinOp::Ge,
                    Rule::plus_infix => BinOp::Add,
                    Rule::minus_infix => BinOp::Sub,
                    Rule::mul => BinOp::Mul,
                    Rule::div => BinOp::Div,
                    Rule::rem => BinOp::Rem,
                    Rule::kh => BinOp::KeepHigh,
                    Rule::kl => BinOp::KeepLow,
                    Rule::rh => BinOp::RemoveHigh,
                    Rule::rl => BinOp::RemoveLow,
                    Rule::repeat_op => BinOp::Repeat,
                    Rule::dice_infix => BinOp::Dice,
                    r => crate::unexpected_rule(r),
                };
                Ok(Expr::Binary(Box::new(BinaryExpr { lhs, op, rhs })))
            })
            .map_postfix(|lhs, op| {
                let lhs = lhs?;
                match op.as_rule() {
                    Rule::call_args => call::build_call_expr(lhs, op, input),
                    Rule::member_access => member_access::build_member_access(lhs, op, input),
                    r => crate::unexpected_rule(r),
                }
            })
            .parse(pair.into_inner()),
        r => crate::unexpected_rule(r),
    }
}

#[cfg(test)]
pub(crate) mod tests {

    use dices_ast::expr::{Expr, scope::ScopeInner};

    use crate::tests::parse_err;

    pub fn expr(expr: Expr) -> ScopeInner {
        ScopeInner {
            statements: vec![],
            expr: Some(expr),
        }
    }

    #[test]
    fn error_unclosed_paren() {
        assert!(parse_err("(1 + 2").is_pest());
    }
}
