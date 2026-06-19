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
    while let Some(op_pair) = inner.next() {
        let op = op_fn(op_pair.as_str());
        let rhs_pair = inner.next().unwrap();
        let rhs = build_expr(rhs_pair, input)?;
        lhs = Expr::Binary(Box::new(BinaryExpr { lhs, op, rhs }));
    }
    Ok(lhs)
}

#[cfg(test)]
pub(crate) mod tests {
    use dices_ast::expr::{
        Expr,
        binary::{BinOp, BinaryExpr},
        unary::UnOp,
    };

    use crate::{
        expr::{tests::expr, unary::tests::unary},
        literal::tests::{bool_val, int, string},
        tests::{parse, parse_err},
    };

    pub fn binary(lhs: Expr, op: BinOp, rhs: Expr) -> Expr {
        Expr::Binary(Box::new(BinaryExpr { lhs, op, rhs }))
    }

    // Binary dice

    #[test]
    fn dice_simple() {
        assert_eq!(parse("3d6"), expr(binary(int("3"), BinOp::Dice, int("6"))));
    }

    #[test]
    fn dice_chained() {
        assert_eq!(
            parse("1d2d3"),
            expr(binary(
                binary(int("1"), BinOp::Dice, int("2")),
                BinOp::Dice,
                int("3")
            ))
        );
    }

    // Repeat

    #[test]
    fn repeat_simple() {
        assert_eq!(
            parse("3d6^2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::Repeat,
                int("2")
            ))
        );
    }

    #[test]
    fn repeat_chained() {
        assert_eq!(
            parse("d6^2^3"),
            expr(binary(
                binary(unary(UnOp::Dice, int("6")), BinOp::Repeat, int("2")),
                BinOp::Repeat,
                int("3")
            ))
        );
    }

    // Arithmetic

    #[test]
    fn arithmetic_add() {
        assert_eq!(parse("1 + 2"), expr(binary(int("1"), BinOp::Add, int("2"))));
    }

    #[test]
    fn arithmetic_sub() {
        assert_eq!(parse("3 - 1"), expr(binary(int("3"), BinOp::Sub, int("1"))));
    }

    #[test]
    fn arithmetic_mul() {
        assert_eq!(parse("2 * 3"), expr(binary(int("2"), BinOp::Mul, int("3"))));
    }

    #[test]
    fn arithmetic_div() {
        assert_eq!(parse("6 / 2"), expr(binary(int("6"), BinOp::Div, int("2"))));
    }

    #[test]
    fn arithmetic_rem() {
        assert_eq!(parse("7 % 3"), expr(binary(int("7"), BinOp::Rem, int("3"))));
    }

    #[test]
    fn arithmetic_chained() {
        assert_eq!(
            parse("1 + 2 + 3"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Add,
                int("3")
            ))
        );
    }

    // Comparison

    #[test]
    fn cmp_eq() {
        assert_eq!(parse("1 == 2"), expr(binary(int("1"), BinOp::Eq, int("2"))));
    }

    #[test]
    fn cmp_ne() {
        assert_eq!(parse("1 != 2"), expr(binary(int("1"), BinOp::Ne, int("2"))));
    }

    #[test]
    fn cmp_lt() {
        assert_eq!(parse("1 < 2"), expr(binary(int("1"), BinOp::Lt, int("2"))));
    }

    #[test]
    fn cmp_gt() {
        assert_eq!(parse("1 > 2"), expr(binary(int("1"), BinOp::Gt, int("2"))));
    }

    #[test]
    fn cmp_le() {
        assert_eq!(parse("1 <= 2"), expr(binary(int("1"), BinOp::Le, int("2"))));
    }

    #[test]
    fn cmp_ge() {
        assert_eq!(parse("1 >= 2"), expr(binary(int("1"), BinOp::Ge, int("2"))));
    }

    // Logic

    #[test]
    fn logic_and() {
        assert_eq!(
            parse("true && false"),
            expr(binary(bool_val(true), BinOp::And, bool_val(false)))
        );
    }

    #[test]
    fn logic_or() {
        assert_eq!(
            parse("true || false"),
            expr(binary(bool_val(true), BinOp::Or, bool_val(false)))
        );
    }

    // Join

    #[test]
    fn join_simple() {
        assert_eq!(
            parse("1 ~ 2"),
            expr(binary(int("1"), BinOp::Join, int("2")))
        );
    }

    #[test]
    fn join_chained() {
        assert_eq!(
            parse("1 ~ 2 ~ 3"),
            expr(binary(
                binary(int("1"), BinOp::Join, int("2")),
                BinOp::Join,
                int("3")
            ))
        );
    }

    // Precedence

    #[test]
    fn precedence_mul_over_add() {
        assert_eq!(
            parse("1 + 2 * 3"),
            expr(binary(
                int("1"),
                BinOp::Add,
                binary(int("2"), BinOp::Mul, int("3"))
            ))
        );
    }

    #[test]
    fn precedence_mul_over_add_right() {
        assert_eq!(
            parse("1 * 2 + 3"),
            expr(binary(
                binary(int("1"), BinOp::Mul, int("2")),
                BinOp::Add,
                int("3")
            ))
        );
    }

    #[test]
    fn precedence_dice_over_repeat() {
        assert_eq!(
            parse("3d6^2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::Repeat,
                int("2")
            ))
        );
    }

    #[test]
    fn precedence_unary_dice_over_repeat() {
        assert_eq!(
            parse("d6^2"),
            expr(binary(unary(UnOp::Dice, int("6")), BinOp::Repeat, int("2")))
        );
    }

    #[test]
    fn precedence_and_over_or() {
        assert_eq!(
            parse("true && false || true"),
            expr(binary(
                binary(bool_val(true), BinOp::And, bool_val(false)),
                BinOp::Or,
                bool_val(true)
            ))
        );
    }

    #[test]
    fn precedence_or_under_and() {
        assert_eq!(
            parse("true || false && true"),
            expr(binary(
                bool_val(true),
                BinOp::Or,
                binary(bool_val(false), BinOp::And, bool_val(true))
            ))
        );
    }

    #[test]
    fn precedence_join_loosest() {
        assert_eq!(
            parse("1 + 2 ~ 3"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Join,
                int("3")
            ))
        );
    }

    #[test]
    fn precedence_cmp_over_add() {
        assert_eq!(
            parse("1 + 2 < 3 + 4"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Lt,
                binary(int("3"), BinOp::Add, int("4"))
            ))
        );
    }

    #[test]
    fn precedence_unary_over_mul() {
        assert_eq!(
            parse("-2 * 3"),
            expr(binary(unary(UnOp::Minus, int("2")), BinOp::Mul, int("3")))
        );
    }

    // Parentheses

    #[test]
    fn parens_override_precedence() {
        assert_eq!(
            parse("(1 + 2) * 3"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Mul,
                int("3")
            ))
        );
    }

    #[test]
    fn parens_nested() {
        assert_eq!(parse("(((42)))"), expr(int("42")));
    }

    #[test]
    fn parens_dice() {
        assert_eq!(
            parse("(1 + 2)d6"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Dice,
                int("6")
            ))
        );
    }

    // Whitespace handling

    #[test]
    fn whitespace_around_operators() {
        assert_eq!(parse("1 + 2"), expr(binary(int("1"), BinOp::Add, int("2"))));
        assert_eq!(parse("1+2"), expr(binary(int("1"), BinOp::Add, int("2"))));
    }

    #[test]
    fn whitespace_around_dice() {
        assert_eq!(
            parse("3 d 6"),
            expr(binary(int("3"), BinOp::Dice, int("6")))
        );
    }

    // Comments

    #[test]
    fn comment_line() {
        assert_eq!(
            parse("1 + // this is a comment\n2"),
            expr(binary(int("1"), BinOp::Add, int("2")))
        );
    }

    #[test]
    fn comment_block() {
        assert_eq!(
            parse("1 /* inline */ + 2"),
            expr(binary(int("1"), BinOp::Add, int("2")))
        );
    }

    #[test]
    fn comment_line_at_end() {
        assert_eq!(parse("42 // trailing comment"), expr(int("42")));
    }

    #[test]
    fn comment_multiline_block() {
        assert_eq!(
            parse("1 /* this spans\n   multiple lines */ + 2"),
            expr(binary(int("1"), BinOp::Add, int("2")))
        );
    }

    #[test]
    fn string_contains_slashes_not_a_comment() {
        assert_eq!(parse("\"hello // world\""), expr(string("hello // world")));
    }

    #[test]
    fn string_contains_block_delimiters_not_a_comment() {
        assert_eq!(
            parse("\"hello /* not a comment */ world\""),
            expr(string("hello /* not a comment */ world"))
        );
    }

    #[test]
    fn comment_before_string() {
        assert_eq!(parse("// comment\n\"hello\""), expr(string("hello")));
    }

    // Misc

    #[test]
    fn three_d_three_is_binary_dice_not_error() {
        assert_eq!(parse("3d3"), expr(binary(int("3"), BinOp::Dice, int("3"))));
    }

    #[test]
    fn error_3dd6() {
        assert!(parse_err("3dd6").is_pest());
    }
}
