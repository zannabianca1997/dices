use dices_ast::expr::{Expr, call::CallExpr};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseCommandError, Rule, expr::build_expr};

pub(super) fn build_call_expr(
    called: Expr,
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Expr, ParseCommandError> {
    let args: Result<Vec<_>, _> = pair.into_inner().map(|p| build_expr(p, input)).collect();
    Ok(Expr::Call(Box::new(CallExpr {
        called,
        args: args?,
    })))
}

#[cfg(test)]
pub(crate) mod tests {
    use dices_ast::expr::{Expr, call::CallExpr};

    use crate::{expr::tests::expr, identifier::ident, literal::tests::int, tests::parse};

    fn call(called: Expr, args: Vec<Expr>) -> Expr {
        Expr::Call(Box::new(CallExpr { called, args }))
    }

    #[test]
    fn no_args() {
        assert_eq!(
            parse("f()"),
            expr(call(Expr::Variable(Box::new(ident("f"))), vec![]))
        );
    }

    #[test]
    fn single_arg() {
        assert_eq!(
            parse("f(42)"),
            expr(call(Expr::Variable(Box::new(ident("f"))), vec![int("42")]))
        );
    }

    #[test]
    fn multiple_args() {
        assert_eq!(
            parse("f(1, 2, 3)"),
            expr(call(
                Expr::Variable(Box::new(ident("f"))),
                vec![int("1"), int("2"), int("3")],
            ))
        );
    }

    #[test]
    fn trailing_comma() {
        assert_eq!(
            parse("f(1,)"),
            expr(call(Expr::Variable(Box::new(ident("f"))), vec![int("1")]))
        );
    }

    #[test]
    fn nested_calls() {
        let inner = parse("f(g(x))");
        assert!(inner.expr.is_some());
    }
}
