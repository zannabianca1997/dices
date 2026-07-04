use std::str::FromStr;

use dices_ast::literal::{LiteralInt, LiteralString};
use dices_values::{int::ValueInt, string::ValueString};
use pest::iterators::Pair;
use snafu::ResultExt;

use crate::{ParseCommandError, Rule};

pub(crate) fn build_string_value(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<LiteralString, ParseCommandError> {
    let span = pair.as_span();
    let range = (span.start() + 1)..(span.end() - 1);
    let s = input
        .slice(range)
        .unwrap()
        .unescape()
        .context(crate::StringUnescapeSnafu)?;
    Ok(LiteralString(s))
}

pub(crate) fn build_int_literal(primary: Pair<'_, Rule>) -> Result<LiteralInt, ParseCommandError> {
    let s = primary.as_str();
    let i = ValueInt::from_str(s).context(crate::IntParseSnafu)?;
    Ok(LiteralInt(i))
}

#[cfg(test)]
pub(crate) mod tests {
    use std::str::FromStr;

    use dices_ast::{
        expr::Expr,
        literal::{Literal, LiteralBool, LiteralInt, LiteralNull, LiteralString},
    };
    use dices_values::{bool::ValueBool, int::ValueInt, null::ValueNull, string::ValueString};

    use crate::{
        expr::tests::expr,
        tests::{parse, parse_err},
    };

    pub fn int(n: &str) -> Expr {
        Expr::Literal(Box::new(Literal::Int(LiteralInt(
            ValueInt::from_str(n).unwrap(),
        ))))
    }

    pub fn string(s: &'static str) -> Expr {
        Expr::Literal(Box::new(Literal::String(LiteralString(
            ValueString::new_static(s),
        ))))
    }

    pub fn bool_val(b: bool) -> Expr {
        Expr::Literal(Box::new(Literal::Bool(LiteralBool(ValueBool::from(b)))))
    }

    pub fn null() -> Expr {
        Expr::Literal(Box::new(Literal::Null(LiteralNull(ValueNull))))
    }

    #[test]
    fn literal_int() {
        assert_eq!(parse("42"), expr(int("42")));
        assert_eq!(parse("0"), expr(int("0")));
        assert_eq!(
            parse("99999999999999999999999999"),
            expr(int("99999999999999999999999999"))
        );
    }

    #[test]
    fn plain() {
        assert_eq!(parse("\"hello\""), expr(string("hello")));
    }

    #[test]
    fn empty() {
        assert_eq!(parse("\"\""), expr(string("")));
    }

    #[test]
    fn escape_newline() {
        assert_eq!(parse("\"hello\\nworld\""), expr(string("hello\nworld")));
    }

    #[test]
    fn escape_tab() {
        assert_eq!(parse("\"hello\\tworld\""), expr(string("hello\tworld")));
    }

    #[test]
    fn escape_quote() {
        assert_eq!(parse("\"hello\\\"world\""), expr(string("hello\"world")));
    }

    #[test]
    fn escape_backslash() {
        assert_eq!(parse("\"hello\\\\world\""), expr(string("hello\\world")));
    }

    #[test]
    fn escape_carriage_return() {
        assert_eq!(parse("\"hello\\rworld\""), expr(string("hello\rworld")));
    }

    #[test]
    fn escape_null() {
        assert_eq!(parse("\"hello\\0world\""), expr(string("hello\0world")));
    }

    #[test]
    fn escape_hex() {
        assert_eq!(parse("\"\\x41\""), expr(string("A")));
    }

    #[test]
    fn escape_unicode() {
        assert_eq!(parse("\"\\u{263A}\""), expr(string("\u{263A}")));
    }

    #[test]
    fn multiple_escapes() {
        assert_eq!(
            parse("\"a\\nb\\tc\\\"d\\\\e\""),
            expr(string("a\nb\tc\"d\\e"))
        );
    }

    #[test]
    fn bool_true() {
        assert_eq!(parse("true"), expr(bool_val(true)));
    }

    #[test]
    fn bool_false() {
        assert_eq!(parse("false"), expr(bool_val(false)));
    }

    #[test]
    fn literal_null() {
        assert_eq!(parse("null"), expr(null()));
    }

    #[test]
    fn error_unclosed_string() {
        assert!(parse_err("\"hello").is_pest());
    }
}
