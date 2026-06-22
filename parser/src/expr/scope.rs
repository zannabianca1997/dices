use dices_ast::{
    expr::{
        Expr,
        scope::{ScopeExpr, ScopeInner},
    },
    statement::Statement,
};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, statement::build_statement};

/// Build a `ScopeInner` from a `scope_inner` or `main` pair.
///
/// The pair's inner consists of `statement_semi*` followed by optional `statement`.
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
            Rule::statement => {
                let stmt = build_statement(inner, input)?;
                match stmt {
                    Statement::Expr(e) => expr = Some(e),
                    Statement::Empty => {}
                    other => statements.push(other),
                }
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
    use dices_ast::{
        expr::{Expr, scope::ScopeInner},
        statement::{Statement, assign::AssignStatement},
    };

    use crate::{identifier::ident, literal::tests::int, tests::parse, tests::parse_err};

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

    #[test]
    fn scope_trailing_semicolon_no_expr() {
        let inner = parse("{ 1; 2; }");
        let Expr::Scope(scope) = inner.expr.as_ref().unwrap() else {
            panic!("expected Scope expr");
        };
        assert_eq!(scope.0.statements.len(), 2);
        assert_eq!(scope.0.expr, None);
    }

    #[test]
    fn scope_assign_no_semicolon() {
        let inner = parse("{ let x = 5 }");
        let Expr::Scope(scope) = inner.expr.as_ref().unwrap() else {
            panic!("expected Scope expr");
        };
        assert_eq!(scope.0.statements.len(), 1);
        assert!(matches!(
            &scope.0.statements[0],
            Statement::Assign(AssignStatement::Let { .. })
        ));
        assert_eq!(scope.0.expr, None);
    }

    #[test]
    fn scope_assign_with_semicolons() {
        let inner = parse("{ let x = 5; let y = 3 }");
        let Expr::Scope(scope) = inner.expr.as_ref().unwrap() else {
            panic!("expected Scope expr");
        };
        assert_eq!(scope.0.statements.len(), 2);
        assert_eq!(scope.0.expr, None);
    }

    #[test]
    fn scope_assign_trailing_semicolon() {
        let inner = parse("{ let x = 5; }");
        let Expr::Scope(scope) = inner.expr.as_ref().unwrap() else {
            panic!("expected Scope expr");
        };
        assert_eq!(scope.0.statements.len(), 1);
        assert_eq!(scope.0.expr, None);
    }

    #[test]
    fn scope_assign_then_expr() {
        let inner = parse("{ let x = 5; x }");
        let Expr::Scope(scope) = inner.expr.as_ref().unwrap() else {
            panic!("expected Scope expr");
        };
        assert_eq!(scope.0.statements.len(), 1);
        assert!(matches!(
            &scope.0.statements[0],
            Statement::Assign(AssignStatement::Let { .. })
        ));
        assert_eq!(
            scope.0.expr,
            Some(Expr::Variable(Box::new(
                ident("x")
            )))
        );
    }

    #[test]
    fn scope_error_assign_no_semicolon_between() {
        assert!(parse_err("{ let x = 5 let y = 3 }").is_pest());
    }
}
