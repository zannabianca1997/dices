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

    /// dd6 is parsed as variable("dd6"), not d(d6)
    ///
    /// Make the grammar easier
    #[test]
    fn dice_nested() {
        // Use d(d6) for nested dice
        assert_eq!(
            parse("d(d6)"),
            expr(unary(UnOp::Dice, unary(UnOp::Dice, int("6"))))
        );
        // dd6 alone is an identifier
        let ident = dices_ast::identifier::Identifier::new(
            dices_values::string::ValueString::new_static("dd6"),
        );
        assert!(ident.is_ok());
    }

    #[test]
    fn chained() {
        assert_eq!(
            parse("-+42"),
            expr(unary(UnOp::Minus, unary(UnOp::Plus, int("42"))))
        );
    }
}
