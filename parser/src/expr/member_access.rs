use pest::iterators::Pair;

use dices_ast::expr::{Expr, member_access::MemberAccessExpr};
use dices_values::string::ValueString;

use crate::{
    Rule,
    expr::{build_expr, map::build_map_key},
    literal::build_int_literal,
};

pub(crate) fn build_member_access(
    container: Expr,
    pair: Pair<'_, Rule>,
    input: &ValueString,
) -> Result<Expr, crate::ParseCommandError> {
    let pair = pair.into_inner().next().unwrap();
    let index = match pair.as_rule() {
        Rule::expr => build_expr(pair, input)?,
        Rule::int => Expr::Literal(Box::new(build_int_literal(pair)?.into())),
        Rule::identifier | Rule::string => {
            Expr::Literal(Box::new(build_map_key(input, pair)?.into()))
        }
        r => crate::unexpected_rule(r),
    };
    Ok(Expr::MemberAccess(Box::new(MemberAccessExpr {
        container,
        index,
    })))
}
