use dices_ast::expr::{
    Expr,
    unary::{UnOp, UnaryExpr},
};
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule, expr::build_expr};

pub(super) fn build_unary(pair: Pair<Rule>, input: &ValueString) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    if first.as_rule() == Rule::filter {
        return build_expr(first, input);
    }
    let op = match first.as_str() {
        "+" => UnOp::Plus,
        "-" => UnOp::Minus,
        "!" => UnOp::Not,
        _ => unreachable!(),
    };
    let operand = build_expr(inner.next().unwrap(), input)?;
    Ok(Expr::Unary(Box::new(UnaryExpr { op, operand })))
}

pub(super) fn build_dice_unary(pair: Pair<Rule>, input: &ValueString) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    if first.as_rule() == Rule::dice_binary {
        return build_expr(first, input);
    }
    let operand = build_expr(inner.next().unwrap(), input)?;
    Ok(Expr::Unary(Box::new(UnaryExpr {
        op: UnOp::Dice,
        operand,
    })))
}

#[cfg(test)]
pub(crate) mod tests {
    use dices_ast::expr::{
        Expr,
        unary::{UnOp, UnaryExpr},
    };

    use crate::{
        expr::tests::expr,
        literal::tests::{bool_val, int},
        tests::parse,
    };

    pub fn unary(op: UnOp, operand: Expr) -> Expr {
        Expr::Unary(Box::new(UnaryExpr { op, operand }))
    }

    #[test]
    fn unary_plus() {
        assert_eq!(parse("+42"), expr(unary(UnOp::Plus, int("42"))));
    }

    #[test]
    fn unary_minus() {
        assert_eq!(parse("-42"), expr(unary(UnOp::Minus, int("42"))));
    }

    #[test]
    fn unary_not() {
        assert_eq!(parse("!true"), expr(unary(UnOp::Not, bool_val(true))));
    }

    #[test]
    fn dice() {
        assert_eq!(parse("d6"), expr(unary(UnOp::Dice, int("6"))));
    }

    #[test]
    fn dice_nested() {
        assert_eq!(
            parse("dd6"),
            expr(unary(UnOp::Dice, unary(UnOp::Dice, int("6"))))
        );
    }

    #[test]
    fn chained() {
        assert_eq!(
            parse("-+42"),
            expr(unary(UnOp::Minus, unary(UnOp::Plus, int("42"))))
        );
    }
}
