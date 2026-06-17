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
            let inner = pair.into_inner().next().unwrap();
            let expr = build_expr(inner, input)?;
            Ok(Statement::Expr(expr))
        }
        r => crate::UnexpectedRuleSnafu { rule: r }.fail(),
    }
}