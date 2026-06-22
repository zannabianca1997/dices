use dices_ast::{
    expr::{Expr, map::MapExpr},
    literal::LiteralString,
};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, expr::build_expr, literal};

pub(super) fn build_map_expr(pair: Pair<Rule>, input: &ValueString) -> Result<Expr, ParseError> {
    let mut pairs = pair.into_inner();
    let mut items = Vec::new();
    while let Some(key_pair) = pairs.next() {
        let value_pair = pairs.next().unwrap();
        let key = match key_pair.as_rule() {
            Rule::string => literal::parse_string_value(key_pair, input)?,
            Rule::identifier => {
                // bare identifier key: use its raw text as the string key (no unescaping needed)
                let span = key_pair.as_span();
                LiteralString(input.slice(span.start()..span.end()).unwrap())
            }
            r => crate::unexpected_rule(r),
        };
        let value = build_expr(value_pair, input)?;
        items.push((key, value));
    }
    Ok(Expr::Map(Box::new(MapExpr { items })))
}

#[cfg(test)]
pub(crate) mod tests {
    use dices_ast::{
        expr::{Expr, map::MapExpr},
        literal::LiteralString,
    };
    use dices_values::string::ValueString;

    use crate::{expr::tests::expr, literal::tests::int, tests::parse};

    fn key(s: &'static str) -> LiteralString {
        LiteralString(ValueString::new_static(s))
    }

    pub fn map(items: Vec<(LiteralString, Expr)>) -> Expr {
        Expr::Map(Box::new(MapExpr { items }))
    }

    #[test]
    fn empty() {
        assert_eq!(parse("<| |>"), expr(map(vec![])));
    }

    #[test]
    fn single_entry() {
        assert_eq!(
            parse(r#"<| "a": 42 |>"#),
            expr(map(vec![(key("a"), int("42"))]))
        );
    }

    #[test]
    fn multiple_entries() {
        assert_eq!(
            parse(r#"<| "a": 1, "b": 2 |>"#),
            expr(map(vec![(key("a"), int("1")), (key("b"), int("2")),]))
        );
    }

    #[test]
    fn trailing_comma() {
        assert_eq!(
            parse(r#"<| "a": 1, |>"#),
            expr(map(vec![(key("a"), int("1"))]))
        );
    }

    #[test]
    fn identifier_key() {
        // a bare identifier key yields the same AST as the quoted string key
        assert_eq!(parse("<| a: 42 |>"), expr(map(vec![(key("a"), int("42"))])));
    }

    #[test]
    fn mixed_keys() {
        assert_eq!(
            parse(r#"<| a: 1, "b": 2 |>"#),
            expr(map(vec![(key("a"), int("1")), (key("b"), int("2")),]))
        );
    }

    #[test]
    fn keyword_identifier_key() {
        // words that are not valid standalone identifiers (e.g. `d`) are still
        // accepted as bare keys
        assert_eq!(parse("<| d: 1 |>"), expr(map(vec![(key("d"), int("1"))])));
    }
}
