use dices_ast::expr::{Expr, list::ListExpr};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseCommandError, Rule, expr::build_expr};

pub(super) fn build_list_expr(pair: Pair<Rule>, input: &ValueString) -> Result<Expr, ParseCommandError> {
    let items: Result<Vec<_>, _> = pair.into_inner().map(|p| build_expr(p, input)).collect();
    Ok(Expr::List(Box::new(ListExpr { items: items? })))
}

#[cfg(test)]
pub(crate) mod tests {
    use dices_ast::expr::{Expr, list::ListExpr};

    use crate::{
        expr::tests::expr,
        literal::tests::{int, string},
        tests::parse,
    };

    pub fn list(items: Vec<Expr>) -> Expr {
        Expr::List(Box::new(ListExpr { items }))
    }

    #[test]
    fn empty() {
        assert_eq!(parse("[]"), expr(list(vec![])));
    }

    #[test]
    fn single() {
        assert_eq!(parse("[42]"), expr(list(vec![int("42")])));
    }

    #[test]
    fn multiple() {
        assert_eq!(
            parse("[1, 2, 3]"),
            expr(list(vec![int("1"), int("2"), int("3")]))
        );
    }

    #[test]
    fn trailing_comma() {
        assert_eq!(parse("[1, 2,]"), expr(list(vec![int("1"), int("2")])));
    }

    #[test]
    fn mixed_expressions() {
        assert_eq!(
            parse(r#"[1, "hello", true]"#),
            expr(list(vec![
                int("1"),
                string("hello"),
                crate::literal::tests::bool_val(true)
            ]))
        );
    }

    #[test]
    fn nested() {
        assert_eq!(
            parse("[[1, 2], [3, 4]]"),
            expr(list(vec![
                list(vec![int("1"), int("2")]),
                list(vec![int("3"), int("4")]),
            ]))
        );
    }
}
