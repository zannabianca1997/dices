use dices_ast::statement::{
    Statement,
    assign::{AssignStatement, Lhs, MemberAccessLhs},
};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{
    ParseCommandError, Rule,
    expr::{build_expr, member_access::build_member_index},
    identifier::parse_identifier,
};

pub(crate) fn build_statement(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Statement, ParseCommandError> {
    match pair.as_rule() {
        Rule::statement => {
            let mut inner = pair.into_inner();
            match inner.next() {
                Some(inner) => match inner.as_rule() {
                    Rule::let_stmt => build_let(inner, input),
                    Rule::set_stmt => build_set(inner, input),
                    Rule::assign => {
                        let assign_inner = inner.into_inner().next().unwrap();
                        match assign_inner.as_rule() {
                            Rule::let_stmt => build_let(assign_inner, input),
                            Rule::set_stmt => build_set(assign_inner, input),
                            r => crate::unexpected_rule(r),
                        }
                    }
                    _ => Ok(Statement::Expr(build_expr(inner, input)?)),
                },
                None => Ok(Statement::Empty),
            }
        }
        Rule::assign => {
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::let_stmt => build_let(inner, input),
                Rule::set_stmt => build_set(inner, input),
                r => crate::unexpected_rule(r),
            }
        }
        r => crate::unexpected_rule(r),
    }
}

fn build_let(pair: Pair<Rule>, input: &ValueString) -> Result<Statement, ParseCommandError> {
    let mut inner = pair.into_inner();
    let ident = parse_identifier(inner.next().unwrap(), input)?;
    let _equals = inner.next().unwrap(); // Rule::equals
    let rhs = build_expr(inner.next().unwrap(), input)?;

    Ok(Statement::Assign(AssignStatement::Let { lhs: ident, rhs }))
}

fn build_set(pair: Pair<Rule>, input: &ValueString) -> Result<Statement, ParseCommandError> {
    let mut inner = pair.into_inner();
    let lhs = build_lhs(inner.next().unwrap(), input)?;
    let _equals = inner.next().unwrap(); // Rule::equals
    let rhs = build_expr(inner.next().unwrap(), input)?;

    Ok(Statement::Assign(AssignStatement::Set { lhs, rhs }))
}

/// Build a [`Rule::lhs`] pair into an [`Lhs`].
///
/// An `lhs` is an `identifier` followed by zero or more `member_access`
/// postfixes. Each postfix is folded into the accumulator, producing a nested
/// [`Lhs::MemberAccess`] chain (e.g. `a.b.c[2]` becomes
/// `a -> .b -> .c -> [2]`).
fn build_lhs(pair: Pair<Rule>, input: &ValueString) -> Result<Lhs, ParseCommandError> {
    let mut inner = pair.into_inner();
    let ident = parse_identifier(inner.next().unwrap(), input)?;
    let mut lhs = Lhs::Variable(ident);
    for member in inner {
        let index = build_member_index(member, input)?;
        lhs = Lhs::MemberAccess(MemberAccessLhs {
            container: Box::new(lhs),
            index,
        });
    }
    Ok(lhs)
}

#[cfg(test)]
mod tests {
    use dices_ast::{
        expr::Expr,
        statement::assign::{AssignStatement, Lhs, MemberAccessLhs},
    };

    use crate::{
        identifier::ident,
        literal::tests::{int, string},
        tests::parse,
    };

    fn set(lhs: Lhs, rhs: Expr) -> AssignStatement {
        AssignStatement::Set { lhs, rhs }
    }

    fn var(name: &'static str) -> Lhs {
        Lhs::Variable(ident(name))
    }

    fn member(container: Lhs, index: Expr) -> Lhs {
        Lhs::MemberAccess(MemberAccessLhs {
            container: Box::new(container),
            index,
        })
    }

    #[test]
    fn plain_variable() {
        let scope = parse("a = 1");
        assert_eq!(scope.statements.len(), 1);
        assert_eq!(scope.statements[0], set(var("a"), int("1")).into());
    }

    #[test]
    fn ident_key() {
        // bare identifier key becomes a string literal index
        let scope = parse("a.b = 1");
        assert_eq!(
            scope.statements[0],
            set(member(var("a"), string("b")), int("1")).into()
        );
    }

    #[test]
    fn string_key() {
        let scope = parse(r#"a."x y" = 1"#);
        assert_eq!(
            scope.statements[0],
            set(member(var("a"), string("x y")), int("1")).into()
        );
    }

    #[test]
    fn int_key() {
        let scope = parse("a.34 = 1");
        assert_eq!(
            scope.statements[0],
            set(member(var("a"), int("34")), int("1")).into()
        );
    }

    #[test]
    fn bracket_expr() {
        let scope = parse("a[1 + 2] = 3");
        let index = Expr::Binary(Box::new(dices_ast::expr::binary::BinaryExpr {
            lhs: int("1"),
            op: dices_ast::expr::binary::BinOp::Add,
            rhs: int("2"),
        }));
        assert_eq!(scope.statements[0], set(member(var("a"), index), int("3")).into());
    }

    #[test]
    fn nested_chain() {
        let scope = parse("a.b.c[2].g = 5");
        // a -> .b -> .c -> [2] -> .g
        let expected = member(
            member(
                member(member(var("a"), string("b")), string("c")),
                int("2"),
            ),
            string("g"),
        );
        assert_eq!(scope.statements[0], set(expected, int("5")).into());
    }

    #[test]
    fn let_stmt_still_uses_plain_identifier() {
        // `let` only accepts a bare name, never a member access
        let scope = parse("let a = 1");
        assert_eq!(
            scope.statements[0],
            AssignStatement::Let {
                lhs: ident("a"),
                rhs: int("1"),
            }
            .into()
        );
    }
}
