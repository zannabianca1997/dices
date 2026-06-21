use dices_ast::expr::{
    Expr,
    scope::{ScopeExpr, ScopeInner},
};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, expr::build_expr, statement::build_statement};

/// Build a `ScopeInner` from a `scope_inner` or `main` pair.
///
/// The pair's inner consists of `statement_semi*` followed by optional `expr`.
pub(crate) fn build_scope_inner(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<ScopeInner, ParseError> {
    let mut statements = Vec::new();
    let mut expr = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::scope_inner => return build_scope_inner(inner, input),
            Rule::statement_semi => {
                let stmt_pair = inner.into_inner().next().unwrap();
                statements.push(build_statement(stmt_pair, input)?);
            }
            Rule::expr => {
                expr = Some(build_expr(inner, input)?);
            }
            r => return crate::UnexpectedRuleSnafu { rule: r }.fail(),
        }
    }

    Ok(ScopeInner { statements, expr })
}

/// Build a scope expression from a `scope` rule pair.
pub(crate) fn build_scope_expr(pair: Pair<Rule>, input: &ValueString) -> Result<Expr, ParseError> {
    let inner = pair.into_inner().next().unwrap();
    let scope_inner = build_scope_inner(inner, input)?;
    Ok(Expr::Scope(Box::new(ScopeExpr(scope_inner))))
}

#[cfg(test)]
pub(crate) mod tests {
    use dices_ast::expr::{Expr, scope::ScopeInner};

    use crate::{literal::tests::int, tests::parse};

    #[test]
    fn scope_single_expr() {
        let inner = parse("{ 42 }");
        assert!(inner.statements.is_empty());
        let Expr::Scope(scope) = inner.expr.as_ref().unwrap() else {
            panic!("expected Scope expr");
        };
        assert_eq!(
            scope.0,
            ScopeInner {
                statements: vec![],
                expr: Some(int("42")),
            }
        );
    }

    #[test]
    fn scope_multiple_stmts() {
        let inner = parse("{ 1; 2; 3 }");
        assert!(inner.statements.is_empty());
        let Expr::Scope(scope) = inner.expr.as_ref().unwrap() else {
            panic!("expected Scope expr");
        };
        assert_eq!(scope.0.statements.len(), 2);
        assert_eq!(scope.0.expr, Some(int("3")));
    }
}
