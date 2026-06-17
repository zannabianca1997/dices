#![doc = include_str!("../README.md")]

use std::str::FromStr;

use dices_ast::expr::Expr;
use dices_values::{
    int::ValueInt,
    string::{EscapeError, ValueString},
};
use pest::Parser;
use pest_derive::Parser;
use snafu::{ResultExt, Snafu};

pub(crate) mod literal;

pub(crate) mod expr;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct Grammar;

#[derive(Debug, Snafu)]
pub enum ParseError {
    #[snafu(display("Pest parse error: {source}"))]
    Pest { source: pest::error::Error<Rule> },
    #[snafu(display("Failed to parse integer: {source}"))]
    IntParse { source: <ValueInt as FromStr>::Err },
    #[snafu(display("Failed to unescape string"))]
    StringUnescape { source: EscapeError },
    #[snafu(display("Unexpected rule: {rule:?}"))]
    UnexpectedRule { rule: Rule },
}

pub fn parse_expr(input: &ValueString) -> Result<Expr, ParseError> {
    let raw = input.as_str();
    let mut pairs = Grammar::parse(Rule::main, raw).context(PestSnafu)?;
    let expr_pair = pairs.next().unwrap();
    expr::build_expr(expr_pair, input)
}

#[cfg(test)]
mod tests {
    use dices_ast::{
        expr::{
            Expr,
            binary::{BinOp, BinaryExpr},
            unary::{UnOp, UnaryExpr},
        },
        literal::Literal,
    };
    use dices_values::{bool::ValueBool, int::ValueInt, null::ValueNull, string::ValueString};
    use std::str::FromStr;

    use super::*;

    fn int(n: &str) -> Expr {
        Expr::Literal(Box::new(Literal::Int(ValueInt::from_str(n).unwrap())))
    }

    fn string(s: &'static str) -> Expr {
        Expr::Literal(Box::new(Literal::String(ValueString::new_static(s))))
    }

    fn bool_val(b: bool) -> Expr {
        Expr::Literal(Box::new(Literal::Bool(ValueBool::from(b))))
    }

    fn null() -> Expr {
        Expr::Literal(Box::new(Literal::Null(ValueNull)))
    }

    fn binary(lhs: Expr, op: BinOp, rhs: Expr) -> Expr {
        Expr::Binary(Box::new(BinaryExpr { lhs, op, rhs }))
    }

    fn unary(op: UnOp, operand: Expr) -> Expr {
        Expr::Unary(Box::new(UnaryExpr { op, operand }))
    }

    fn parse(input: &'static str) -> Expr {
        parse_expr(&ValueString::new_static(input)).unwrap()
    }

    fn parse_err(input: &'static str) -> ParseError {
        parse_expr(&ValueString::new_static(input)).unwrap_err()
    }

    // ── Literals ──────────────────────────────────────────────────────

    #[test]
    fn literal_int() {
        assert_eq!(parse("42"), int("42"));
        assert_eq!(parse("0"), int("0"));
        assert_eq!(
            parse("99999999999999999999999999"),
            int("99999999999999999999999999")
        );
    }

    #[test]
    fn literal_string_plain() {
        assert_eq!(parse("\"hello\""), string("hello"));
    }

    #[test]
    fn literal_string_empty() {
        assert_eq!(parse("\"\""), string(""));
    }

    #[test]
    fn literal_string_escape_newline() {
        assert_eq!(parse("\"hello\\nworld\""), string("hello\nworld"));
    }

    #[test]
    fn literal_string_escape_tab() {
        assert_eq!(parse("\"hello\\tworld\""), string("hello\tworld"));
    }

    #[test]
    fn literal_string_escape_quote() {
        assert_eq!(parse("\"hello\\\"world\""), string("hello\"world"));
    }

    #[test]
    fn literal_string_escape_backslash() {
        assert_eq!(parse("\"hello\\\\world\""), string("hello\\world"));
    }

    #[test]
    fn literal_string_escape_carriage_return() {
        assert_eq!(parse("\"hello\\rworld\""), string("hello\rworld"));
    }

    #[test]
    fn literal_string_escape_null() {
        assert_eq!(parse("\"hello\\0world\""), string("hello\0world"));
    }

    #[test]
    fn literal_string_escape_hex() {
        assert_eq!(parse("\"\\x41\""), string("A"));
    }

    #[test]
    fn literal_string_escape_unicode() {
        assert_eq!(parse("\"\\u{263A}\""), string("\u{263A}"));
    }

    #[test]
    fn literal_string_multiple_escapes() {
        assert_eq!(parse("\"a\\nb\\tc\\\"d\\\\e\""), string("a\nb\tc\"d\\e"));
    }

    #[test]
    fn literal_bool_true() {
        assert_eq!(parse("true"), bool_val(true));
    }

    #[test]
    fn literal_bool_false() {
        assert_eq!(parse("false"), bool_val(false));
    }

    #[test]
    fn literal_null() {
        assert_eq!(parse("null"), null());
    }

    // ── Unary operators ───────────────────────────────────────────────

    #[test]
    fn unary_plus() {
        assert_eq!(parse("+42"), unary(UnOp::Plus, int("42")));
    }

    #[test]
    fn unary_minus() {
        assert_eq!(parse("-42"), unary(UnOp::Minus, int("42")));
    }

    #[test]
    fn unary_not() {
        assert_eq!(parse("!true"), unary(UnOp::Not, bool_val(true)));
    }

    #[test]
    fn unary_dice() {
        assert_eq!(parse("d6"), unary(UnOp::Dice, int("6")));
    }

    #[test]
    fn unary_dice_nested() {
        assert_eq!(parse("dd6"), unary(UnOp::Dice, unary(UnOp::Dice, int("6"))));
    }

    #[test]
    fn unary_chained() {
        assert_eq!(
            parse("-+42"),
            unary(UnOp::Minus, unary(UnOp::Plus, int("42")))
        );
    }

    // ── Binary dice ───────────────────────────────────────────────────

    #[test]
    fn binary_dice_simple() {
        assert_eq!(parse("3d6"), binary(int("3"), BinOp::Dice, int("6")));
    }

    #[test]
    fn binary_dice_chained() {
        assert_eq!(
            parse("1d2d3"),
            binary(
                binary(int("1"), BinOp::Dice, int("2")),
                BinOp::Dice,
                int("3")
            )
        );
    }

    // ── Repeat ────────────────────────────────────────────────────────

    #[test]
    fn repeat_simple() {
        assert_eq!(
            parse("3d6^2"),
            binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::Repeat,
                int("2")
            )
        );
    }

    #[test]
    fn repeat_chained() {
        assert_eq!(
            parse("d6^2^3"),
            binary(
                binary(unary(UnOp::Dice, int("6")), BinOp::Repeat, int("2")),
                BinOp::Repeat,
                int("3")
            )
        );
    }

    // ── Arithmetic ────────────────────────────────────────────────────

    #[test]
    fn arithmetic_add() {
        assert_eq!(parse("1 + 2"), binary(int("1"), BinOp::Add, int("2")));
    }

    #[test]
    fn arithmetic_sub() {
        assert_eq!(parse("3 - 1"), binary(int("3"), BinOp::Sub, int("1")));
    }

    #[test]
    fn arithmetic_mul() {
        assert_eq!(parse("2 * 3"), binary(int("2"), BinOp::Mul, int("3")));
    }

    #[test]
    fn arithmetic_div() {
        assert_eq!(parse("6 / 2"), binary(int("6"), BinOp::Div, int("2")));
    }

    #[test]
    fn arithmetic_rem() {
        assert_eq!(parse("7 % 3"), binary(int("7"), BinOp::Rem, int("3")));
    }

    #[test]
    fn arithmetic_chained() {
        assert_eq!(
            parse("1 + 2 + 3"),
            binary(binary(int("1"), BinOp::Add, int("2")), BinOp::Add, int("3"))
        );
    }

    // ── Comparison ────────────────────────────────────────────────────

    #[test]
    fn cmp_eq() {
        assert_eq!(parse("1 == 2"), binary(int("1"), BinOp::Eq, int("2")));
    }

    #[test]
    fn cmp_ne() {
        assert_eq!(parse("1 != 2"), binary(int("1"), BinOp::Ne, int("2")));
    }

    #[test]
    fn cmp_lt() {
        assert_eq!(parse("1 < 2"), binary(int("1"), BinOp::Lt, int("2")));
    }

    #[test]
    fn cmp_gt() {
        assert_eq!(parse("1 > 2"), binary(int("1"), BinOp::Gt, int("2")));
    }

    #[test]
    fn cmp_le() {
        assert_eq!(parse("1 <= 2"), binary(int("1"), BinOp::Le, int("2")));
    }

    #[test]
    fn cmp_ge() {
        assert_eq!(parse("1 >= 2"), binary(int("1"), BinOp::Ge, int("2")));
    }

    // ── Logic ─────────────────────────────────────────────────────────

    #[test]
    fn logic_and() {
        assert_eq!(
            parse("true && false"),
            binary(bool_val(true), BinOp::And, bool_val(false))
        );
    }

    #[test]
    fn logic_or() {
        assert_eq!(
            parse("true || false"),
            binary(bool_val(true), BinOp::Or, bool_val(false))
        );
    }

    // ── Join ──────────────────────────────────────────────────────────

    #[test]
    fn join_simple() {
        assert_eq!(parse("1 ~ 2"), binary(int("1"), BinOp::Join, int("2")));
    }

    #[test]
    fn join_chained() {
        assert_eq!(
            parse("1 ~ 2 ~ 3"),
            binary(
                binary(int("1"), BinOp::Join, int("2")),
                BinOp::Join,
                int("3")
            )
        );
    }

    // ── Precedence ────────────────────────────────────────────────────

    #[test]
    fn precedence_mul_over_add() {
        assert_eq!(
            parse("1 + 2 * 3"),
            binary(int("1"), BinOp::Add, binary(int("2"), BinOp::Mul, int("3")))
        );
    }

    #[test]
    fn precedence_mul_over_add_right() {
        assert_eq!(
            parse("1 * 2 + 3"),
            binary(binary(int("1"), BinOp::Mul, int("2")), BinOp::Add, int("3"))
        );
    }

    #[test]
    fn precedence_dice_over_repeat() {
        assert_eq!(
            parse("3d6^2"),
            binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::Repeat,
                int("2")
            )
        );
    }

    #[test]
    fn precedence_unary_dice_over_repeat() {
        assert_eq!(
            parse("d6^2"),
            binary(unary(UnOp::Dice, int("6")), BinOp::Repeat, int("2"))
        );
    }

    #[test]
    fn precedence_and_over_or() {
        assert_eq!(
            parse("true && false || true"),
            binary(
                binary(bool_val(true), BinOp::And, bool_val(false)),
                BinOp::Or,
                bool_val(true)
            )
        );
    }

    #[test]
    fn precedence_or_under_and() {
        assert_eq!(
            parse("true || false && true"),
            binary(
                bool_val(true),
                BinOp::Or,
                binary(bool_val(false), BinOp::And, bool_val(true))
            )
        );
    }

    #[test]
    fn precedence_join_loosest() {
        assert_eq!(
            parse("1 + 2 ~ 3"),
            binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Join,
                int("3")
            )
        );
    }

    #[test]
    fn precedence_cmp_over_add() {
        assert_eq!(
            parse("1 + 2 < 3 + 4"),
            binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Lt,
                binary(int("3"), BinOp::Add, int("4"))
            )
        );
    }

    #[test]
    fn precedence_unary_over_mul() {
        assert_eq!(
            parse("-2 * 3"),
            binary(unary(UnOp::Minus, int("2")), BinOp::Mul, int("3"))
        );
    }

    // ── Parentheses ───────────────────────────────────────────────────

    #[test]
    fn parens_override_precedence() {
        assert_eq!(
            parse("(1 + 2) * 3"),
            binary(binary(int("1"), BinOp::Add, int("2")), BinOp::Mul, int("3"))
        );
    }

    #[test]
    fn parens_nested() {
        assert_eq!(parse("(((42)))"), int("42"));
    }

    #[test]
    fn parens_dice() {
        assert_eq!(
            parse("(1 + 2)d6"),
            binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Dice,
                int("6")
            )
        );
    }

    // ── Whitespace handling ───────────────────────────────────────────

    #[test]
    fn whitespace_around_operators() {
        assert_eq!(parse("1 + 2"), binary(int("1"), BinOp::Add, int("2")));
        assert_eq!(parse("1+2"), binary(int("1"), BinOp::Add, int("2")));
    }

    #[test]
    fn whitespace_around_dice() {
        assert_eq!(parse("3 d 6"), binary(int("3"), BinOp::Dice, int("6")));
    }

    // ── Comments ───────────────────────────────────────────────────────

    #[test]
    fn comment_line() {
        assert_eq!(
            parse("1 + // this is a comment\n2"),
            binary(int("1"), BinOp::Add, int("2"))
        );
    }

    #[test]
    fn comment_block() {
        assert_eq!(
            parse("1 /* inline */ + 2"),
            binary(int("1"), BinOp::Add, int("2"))
        );
    }

    #[test]
    fn comment_line_at_end() {
        assert_eq!(parse("42 // trailing comment"), int("42"));
    }

    #[test]
    fn comment_multiline_block() {
        assert_eq!(
            parse("1 /* this spans\n   multiple lines */ + 2"),
            binary(int("1"), BinOp::Add, int("2"))
        );
    }

    #[test]
    fn string_contains_slashes_not_a_comment() {
        // `//` inside a string must not start a line comment
        assert_eq!(parse("\"hello // world\""), string("hello // world"));
    }

    #[test]
    fn string_contains_block_delimiters_not_a_comment() {
        // `/* */` inside a string must not start a block comment
        assert_eq!(
            parse("\"hello /* not a comment */ world\""),
            string("hello /* not a comment */ world")
        );
    }

    #[test]
    fn comment_before_string() {
        assert_eq!(
            parse("// comment\n\"hello\""),
            string("hello")
        );
    }

    // ── Error cases ───────────────────────────────────────────────────

    #[test]
    fn error_empty_input() {
        assert!(parse_err("").to_string().contains("Pest"));
    }

    #[test]
    fn error_garbage() {
        assert!(parse_err("foo bar").to_string().contains("Pest"));
    }

    #[test]
    fn error_unclosed_paren() {
        assert!(parse_err("(1 + 2").to_string().contains("Pest"));
    }

    #[test]
    fn error_unclosed_string() {
        assert!(parse_err("\"hello").to_string().contains("Pest"));
    }

    #[test]
    fn error_3dd6() {
        // `3dd6` should fail: `3d` consumes `3` and `d`, then `d6` is not a valid atom
        assert!(parse_err("3dd6").to_string().contains("Pest"));
    }

    #[test]
    fn error_3d3_is_binary_dice_not_error() {
        // `3d3` should be Binary dice, not 3(d3) which would fail
        assert_eq!(parse("3d3"), binary(int("3"), BinOp::Dice, int("3")));
    }
}
