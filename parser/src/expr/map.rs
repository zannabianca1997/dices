use dices_ast::expr::{Expr, map::MapExpr};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, expr::build_expr, literal};

pub(super) fn build_map_expr(pair: Pair<Rule>, input: &ValueString) -> Result<Expr, ParseError> {
    let mut pairs = pair.into_inner();
    let mut items = Vec::new();
    while let Some(key_pair) = pairs.next() {
        let value_pair = pairs.next().unwrap();
        let key = literal::parse_string_value(key_pair, input)?;
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
}
