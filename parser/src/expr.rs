use dices_ast::expr::Expr;
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{
    ParseError, Rule,
    literal,
};

pub(crate) mod binary;
pub(crate) mod unary;

pub(crate) fn build_expr(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Expr, ParseError> {
    use dices_ast::expr::binary::BinOp;

    match pair.as_rule() {
        Rule::main => {
            let inner = pair.into_inner().next().unwrap();
            build_expr(inner, input)
        }
        Rule::expr => {
            let inner = pair.into_inner().next().unwrap();
            build_expr(inner, input)
        }
        Rule::join => binary::build_binary_chain(pair, BinOp::Join, input),
        Rule::or => binary::build_binary_chain(pair, BinOp::Or, input),
        Rule::and => binary::build_binary_chain(pair, BinOp::And, input),
        Rule::cmp => binary::build_operator_chain(pair, binary::cmp_op_to_binop, input),
        Rule::add => binary::build_operator_chain(pair, binary::add_op_to_binop, input),
        Rule::mul => {
            binary::build_operator_chain(pair, binary::mul_op_to_binop, input)
        }
        Rule::unary => unary::build_unary(pair, input),
        Rule::repeat => binary::build_binary_chain(pair, BinOp::Repeat, input),
        Rule::dice_unary => unary::build_dice_unary(pair, input),
        Rule::dice_binary => binary::build_binary_chain(pair, BinOp::Dice, input),
        Rule::atom => {
            let inner = pair.into_inner().next().unwrap();
            build_expr(inner, input)
        }
        Rule::literal => literal::build_literal(pair, input),
        Rule::int => literal::build_int(pair),
        Rule::string => literal::build_string(pair, input),
        Rule::bool => literal::build_bool(pair),
        Rule::null => Ok(literal::build_null(pair)),
        r => crate::UnexpectedRuleSnafu { rule: r }.fail(),
    }
}