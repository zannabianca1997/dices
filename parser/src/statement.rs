use dices_ast::statement::Statement;
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, expr::build_expr};

pub(crate) fn build_statement(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Statement, ParseError> {
    match pair.as_rule() {
        Rule::main => {
            let inner = pair.into_inner().next().unwrap();
            build_statement(inner, input)
        }
        Rule::statement => {
            let mut inner = pair.into_inner();
            match inner.next() {
                Some(inner) => {
                    let expr = build_expr(inner, input)?;
                    Ok(Statement::Expr(expr))
                }
                None => Ok(Statement::Empty),
            }
        }
        r => crate::UnexpectedRuleSnafu { rule: r }.fail(),
    }
}
