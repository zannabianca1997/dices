use dices_ast::statement::{
    Statement,
    assign::{AssignStatement, Lhs},
};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseCommandError, Rule, expr::build_expr, identifier::parse_identifier};

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
    let ident = parse_identifier(inner.next().unwrap(), input)?;
    let _equals = inner.next().unwrap(); // Rule::equals
    let rhs = build_expr(inner.next().unwrap(), input)?;

    Ok(Statement::Assign(AssignStatement::Set {
        lhs: Lhs::Variable(ident),
        rhs,
    }))
}
