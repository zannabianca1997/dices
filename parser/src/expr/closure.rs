use std::sync::Arc;

use dices_ast::expr::{Expr, closure::ClosureExpr};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, identifier::parse_identifier};

pub(super) fn build_closure_expr(
    pair: Pair<Rule>,
    body: Expr,
    input: &ValueString,
) -> Result<Expr, ParseError> {
    let args: Result<Vec<_>, _> = pair
        .into_inner()
        .map(|p| parse_identifier(p, input))
        .collect();
    Ok(Expr::Closure(Arc::new(ClosureExpr { args: args?, body })))
}

#[cfg(test)]
pub(crate) mod tests {
    use std::sync::Arc;

    use dices_ast::expr::{Expr, closure::ClosureExpr};

    use crate::{
        expr::tests::expr, identifier::ident, literal::tests::int, tests::parse,
    };

    fn closure(args: Vec<&'static str>, body: Expr) -> Expr {
        Expr::Closure(Arc::new(ClosureExpr {
            args: args.into_iter().map(ident).collect(),
            body,
        }))
    }

    #[test]
    fn single_arg() {
        assert_eq!(
            parse("|x| x"),
            expr(closure(vec!["x"], Expr::Variable(Box::new(ident("x")))))
        );
    }

    #[test]
    fn multiple_args() {
        assert_eq!(
            parse("|x, y| x"),
            expr(closure(
                vec!["x", "y"],
                Expr::Variable(Box::new(ident("x"))),
            ))
        );
    }

    #[test]
    fn no_args() {
        assert_eq!(parse("|| 42"), expr(closure(vec![], int("42"))));
    }

    #[test]
    fn body_literal() {
        assert_eq!(parse("|x| 42"), expr(closure(vec!["x"], int("42"))));
    }

    #[test]
    fn body_scope() {
        let inner = parse("|x| { x }");
        assert!(inner.expr.is_some());
    }
}
