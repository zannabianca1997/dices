#[cfg(test)]
pub(crate) mod tests {
    use dices_ast::expr::{
        Expr,
        binary::{BinOp, BinaryExpr},
        unary::UnOp,
    };

    use crate::{
        expr::{tests::expr, unary::tests::unary},
        literal::tests::{bool_val, int, string},
        tests::{parse, parse_err},
    };

    pub fn binary(lhs: Expr, op: BinOp, rhs: Expr) -> Expr {
        Expr::Binary(Box::new(BinaryExpr { lhs, op, rhs }))
    }

    // Binary dice

    #[test]
    fn dice_simple() {
        assert_eq!(parse("3d6"), expr(binary(int("3"), BinOp::Dice, int("6"))));
    }

    #[test]
    fn dice_chained() {
        assert_eq!(
            parse("1d2d3"),
            expr(binary(
                binary(int("1"), BinOp::Dice, int("2")),
                BinOp::Dice,
                int("3")
            ))
        );
    }

    // Repeat

    #[test]
    fn repeat_simple() {
        assert_eq!(
            parse("3d6^2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::Repeat,
                int("2")
            ))
        );
    }

    #[test]
    fn repeat_chained() {
        assert_eq!(
            parse("d6^2^3"),
            expr(binary(
                binary(unary(UnOp::Dice, int("6")), BinOp::Repeat, int("2")),
                BinOp::Repeat,
                int("3")
            ))
        );
    }

    // Arithmetic

    #[test]
    fn arithmetic_add() {
        assert_eq!(parse("1 + 2"), expr(binary(int("1"), BinOp::Add, int("2"))));
    }

    #[test]
    fn arithmetic_sub() {
        assert_eq!(parse("3 - 1"), expr(binary(int("3"), BinOp::Sub, int("1"))));
    }

    #[test]
    fn arithmetic_mul() {
        assert_eq!(parse("2 * 3"), expr(binary(int("2"), BinOp::Mul, int("3"))));
    }

    #[test]
    fn arithmetic_div() {
        assert_eq!(parse("6 / 2"), expr(binary(int("6"), BinOp::Div, int("2"))));
    }

    #[test]
    fn arithmetic_rem() {
        assert_eq!(parse("7 % 3"), expr(binary(int("7"), BinOp::Rem, int("3"))));
    }

    #[test]
    fn arithmetic_chained() {
        assert_eq!(
            parse("1 + 2 + 3"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Add,
                int("3")
            ))
        );
    }

    // Comparison

    #[test]
    fn cmp_eq() {
        assert_eq!(parse("1 == 2"), expr(binary(int("1"), BinOp::Eq, int("2"))));
    }

    #[test]
    fn cmp_ne() {
        assert_eq!(parse("1 != 2"), expr(binary(int("1"), BinOp::Ne, int("2"))));
    }

    #[test]
    fn cmp_lt() {
        assert_eq!(parse("1 < 2"), expr(binary(int("1"), BinOp::Lt, int("2"))));
    }

    #[test]
    fn cmp_gt() {
        assert_eq!(parse("1 > 2"), expr(binary(int("1"), BinOp::Gt, int("2"))));
    }

    #[test]
    fn cmp_le() {
        assert_eq!(parse("1 <= 2"), expr(binary(int("1"), BinOp::Le, int("2"))));
    }

    #[test]
    fn cmp_ge() {
        assert_eq!(parse("1 >= 2"), expr(binary(int("1"), BinOp::Ge, int("2"))));
    }

    // Logic

    #[test]
    fn logic_and() {
        assert_eq!(
            parse("true && false"),
            expr(binary(bool_val(true), BinOp::And, bool_val(false)))
        );
    }

    #[test]
    fn logic_or() {
        assert_eq!(
            parse("true || false"),
            expr(binary(bool_val(true), BinOp::Or, bool_val(false)))
        );
    }

    // Join

    #[test]
    fn join_simple() {
        assert_eq!(
            parse("1 ~ 2"),
            expr(binary(int("1"), BinOp::Join, int("2")))
        );
    }

    #[test]
    fn join_chained() {
        assert_eq!(
            parse("1 ~ 2 ~ 3"),
            expr(binary(
                binary(int("1"), BinOp::Join, int("2")),
                BinOp::Join,
                int("3")
            ))
        );
    }

    // Precedence

    #[test]
    fn precedence_mul_over_add() {
        assert_eq!(
            parse("1 + 2 * 3"),
            expr(binary(
                int("1"),
                BinOp::Add,
                binary(int("2"), BinOp::Mul, int("3"))
            ))
        );
    }

    #[test]
    fn precedence_mul_over_add_right() {
        assert_eq!(
            parse("1 * 2 + 3"),
            expr(binary(
                binary(int("1"), BinOp::Mul, int("2")),
                BinOp::Add,
                int("3")
            ))
        );
    }

    #[test]
    fn precedence_dice_over_repeat() {
        assert_eq!(
            parse("3d6^2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::Repeat,
                int("2")
            ))
        );
    }

    #[test]
    fn precedence_unary_dice_over_repeat() {
        assert_eq!(
            parse("d6^2"),
            expr(binary(unary(UnOp::Dice, int("6")), BinOp::Repeat, int("2")))
        );
    }

    #[test]
    fn precedence_and_over_or() {
        assert_eq!(
            parse("true && false || true"),
            expr(binary(
                binary(bool_val(true), BinOp::And, bool_val(false)),
                BinOp::Or,
                bool_val(true)
            ))
        );
    }

    #[test]
    fn precedence_or_under_and() {
        assert_eq!(
            parse("true || false && true"),
            expr(binary(
                bool_val(true),
                BinOp::Or,
                binary(bool_val(false), BinOp::And, bool_val(true))
            ))
        );
    }

    #[test]
    fn precedence_join_loosest() {
        assert_eq!(
            parse("1 + 2 ~ 3"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Join,
                int("3")
            ))
        );
    }

    #[test]
    fn precedence_cmp_over_add() {
        assert_eq!(
            parse("1 + 2 < 3 + 4"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Lt,
                binary(int("3"), BinOp::Add, int("4"))
            ))
        );
    }

    #[test]
    fn precedence_unary_over_mul() {
        assert_eq!(
            parse("-2 * 3"),
            expr(binary(unary(UnOp::Minus, int("2")), BinOp::Mul, int("3")))
        );
    }

    // Parentheses

    #[test]
    fn parens_override_precedence() {
        assert_eq!(
            parse("(1 + 2) * 3"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Mul,
                int("3")
            ))
        );
    }

    #[test]
    fn parens_nested() {
        assert_eq!(parse("(((42)))"), expr(int("42")));
    }

    #[test]
    fn parens_dice() {
        assert_eq!(
            parse("(1 + 2)d6"),
            expr(binary(
                binary(int("1"), BinOp::Add, int("2")),
                BinOp::Dice,
                int("6")
            ))
        );
    }

    // Whitespace handling

    #[test]
    fn whitespace_around_operators() {
        assert_eq!(parse("1 + 2"), expr(binary(int("1"), BinOp::Add, int("2"))));
        assert_eq!(parse("1+2"), expr(binary(int("1"), BinOp::Add, int("2"))));
    }

    #[test]
    fn whitespace_around_dice() {
        assert_eq!(
            parse("3 d 6"),
            expr(binary(int("3"), BinOp::Dice, int("6")))
        );
    }

    // Comments

    #[test]
    fn comment_line() {
        assert_eq!(
            parse("1 + // this is a comment\n2"),
            expr(binary(int("1"), BinOp::Add, int("2")))
        );
    }

    #[test]
    fn comment_block() {
        assert_eq!(
            parse("1 /* inline */ + 2"),
            expr(binary(int("1"), BinOp::Add, int("2")))
        );
    }

    #[test]
    fn comment_line_at_end() {
        assert_eq!(parse("42 // trailing comment"), expr(int("42")));
    }

    #[test]
    fn comment_multiline_block() {
        assert_eq!(
            parse("1 /* this spans\n   multiple lines */ + 2"),
            expr(binary(int("1"), BinOp::Add, int("2")))
        );
    }

    #[test]
    fn string_contains_slashes_not_a_comment() {
        assert_eq!(parse("\"hello // world\""), expr(string("hello // world")));
    }

    #[test]
    fn string_contains_block_delimiters_not_a_comment() {
        assert_eq!(
            parse("\"hello /* not a comment */ world\""),
            expr(string("hello /* not a comment */ world"))
        );
    }

    #[test]
    fn comment_before_string() {
        assert_eq!(parse("// comment\n\"hello\""), expr(string("hello")));
    }

    // Misc

    #[test]
    fn three_d_three_is_binary_dice_not_error() {
        assert_eq!(parse("3d3"), expr(binary(int("3"), BinOp::Dice, int("3"))));
    }

    #[test]
    fn error_3dd6() {
        // 3dd6 now fails at the pest level: dice_infix lookahead fails on second 'd',
        // leaving unconsumed input
        assert!(parse_err("3dd6").is_pest());
    }

    // Filter operators

    #[test]
    fn filter_keep_high() {
        assert_eq!(
            parse("3d6kh2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::KeepHigh,
                int("2")
            ))
        );
    }

    #[test]
    fn filter_keep_low() {
        assert_eq!(
            parse("3d6kl2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::KeepLow,
                int("2")
            ))
        );
    }

    #[test]
    fn filter_remove_high() {
        assert_eq!(
            parse("3d6rh2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::RemoveHigh,
                int("2")
            ))
        );
    }

    #[test]
    fn filter_remove_low() {
        assert_eq!(
            parse("3d6rl2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::RemoveLow,
                int("2")
            ))
        );
    }

    #[test]
    fn filter_keep_high_list() {
        assert_eq!(
            parse("[1, 2, 3] kh 2"),
            expr(binary(
                Expr::List(Box::new(dices_ast::expr::list::ListExpr {
                    items: vec![int("1"), int("2"), int("3")],
                })),
                BinOp::KeepHigh,
                int("2")
            ))
        );
    }

    #[test]
    fn filter_chain_repeat_under_filter() {
        // 3d6kh2^3 => 3d6 kh (2^3)
        assert_eq!(
            parse("3d6kh2^3"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::KeepHigh,
                binary(int("2"), BinOp::Repeat, int("3"))
            ))
        );
    }

    #[test]
    fn filter_chain_repeat_over_filter() {
        // (3d6kh2)^3 => repeat keep-high result
        assert_eq!(
            parse("(3d6kh2)^3"),
            expr(binary(
                binary(
                    binary(int("3"), BinOp::Dice, int("6")),
                    BinOp::KeepHigh,
                    int("2")
                ),
                BinOp::Repeat,
                int("3")
            ))
        );
    }

    #[test]
    fn filter_binds_tighter_than_add() {
        assert_eq!(
            parse("3d6kh2 + 1"),
            expr(binary(
                binary(
                    binary(int("3"), BinOp::Dice, int("6")),
                    BinOp::KeepHigh,
                    int("2")
                ),
                BinOp::Add,
                int("1")
            ))
        );
    }

    #[test]
    fn filter_looser_than_repeat() {
        // 3d6^2kh2 => (3d6^2) kh 2
        assert_eq!(
            parse("3d6^2kh2"),
            expr(binary(
                binary(
                    binary(int("3"), BinOp::Dice, int("6")),
                    BinOp::Repeat,
                    int("2")
                ),
                BinOp::KeepHigh,
                int("2")
            ))
        );
    }

    #[test]
    fn filter_tighter_than_repeat_reversed() {
        // 3d6kh2^2 => 3d6 kh (2^2)
        assert_eq!(
            parse("3d6kh2^2"),
            expr(binary(
                binary(int("3"), BinOp::Dice, int("6")),
                BinOp::KeepHigh,
                binary(int("2"), BinOp::Repeat, int("2"))
            ))
        );
    }

    // Variables

    fn variable(name: &'static str) -> Expr {
        let ident = dices_ast::identifier::Identifier::new(
            dices_values::string::ValueString::new_static(name),
        )
        .unwrap();
        Expr::Variable(Box::new(ident))
    }

    #[test]
    fn variable_simple() {
        assert_eq!(parse("x"), expr(variable("x")));
    }

    #[test]
    fn variable_with_underscores() {
        assert_eq!(parse("_foo"), expr(variable("_foo")));
    }

    #[test]
    fn variable_in_expression() {
        assert_eq!(
            parse("x + 1"),
            expr(binary(variable("x"), BinOp::Add, int("1")))
        );
    }

    #[test]
    fn variable_passed_to_filter() {
        assert_eq!(
            parse("x kh 2"),
            expr(binary(variable("x"), BinOp::KeepHigh, int("2")))
        );
    }

    #[test]
    fn variable_dice_is_valid() {
        assert_eq!(parse("dice"), expr(variable("dice")));
    }

    #[test]
    fn variable_dkh_is_valid() {
        assert_eq!(parse("dkh"), expr(variable("dkh")));
    }

    #[test]
    fn tricky_2d20kh1_parses_as_dice_expr() {
        // 2d20kh1 => (2 d 20) kh 1, NOT invalid identifier
        assert_eq!(
            parse("2d20kh1"),
            expr(binary(
                binary(int("2"), BinOp::Dice, int("20")),
                BinOp::KeepHigh,
                int("1")
            ))
        );
    }

    #[test]
    fn tricky_d20kh1_parses_as_unary_dice() {
        // d20kh1 => (d 20) kh 1, NOT invalid identifier
        assert_eq!(
            parse("d20kh1"),
            expr(binary(
                unary(UnOp::Dice, int("20")),
                BinOp::KeepHigh,
                int("1")
            ))
        );
    }

    #[test]
    fn d20_parses_as_unary_dice() {
        // d20 is a valid dice expression, not an invalid identifier
        assert_eq!(parse("d20"), expr(unary(UnOp::Dice, int("20"))));
    }

    #[test]
    fn error_3d_requires_right_operand() {
        // 3d fails because 'd' expects a right operand
        assert!(parse_err("3d").is_pest());
    }

    #[test]
    fn rhs_is_a_valid_identifier() {
        // rhs is a valid identifier, not invalid
        assert_eq!(parse("rhs"), expr(variable("rhs")));
    }

    #[test]
    fn error_let_is_keyword() {
        assert!(parse_err("let + 1").is_invalid_identifier() || parse_err("let + 1").is_pest());
    }

    // Assignments

    use dices_ast::identifier::Identifier;
    use dices_ast::statement::{
        Statement,
        assign::{AssignStatement, Lhs},
    };
    use dices_values::string::ValueString;

    #[test]
    fn assignment_let() {
        let inner = parse("let x = 5");
        assert_eq!(inner.expr, None);
        assert_eq!(inner.statements.len(), 1);
        let stmt = &inner.statements[0];
        let Statement::Assign(AssignStatement::Let { lhs, rhs }) = stmt else {
            panic!("expected Let statement, got {stmt:?}");
        };
        assert_eq!(lhs, &Identifier::new(ValueString::new_static("x")).unwrap());
        assert_eq!(rhs, &int("5"));
    }

    #[test]
    fn assignment_set() {
        let inner = parse("x = 5");
        assert_eq!(inner.expr, None);
        assert_eq!(inner.statements.len(), 1);
        let stmt = &inner.statements[0];
        let Statement::Assign(AssignStatement::Set { lhs, rhs }) = stmt else {
            panic!("expected Set statement, got {stmt:?}");
        };
        assert_eq!(
            *lhs,
            Lhs::Variable(Identifier::new(ValueString::new_static("x")).unwrap())
        );
        assert_eq!(rhs, &int("5"));
    }

    #[test]
    fn assignment_let_with_expr() {
        let inner = parse("let x = 1 + 2");
        assert_eq!(inner.statements.len(), 1);
        let stmt = &inner.statements[0];
        let Statement::Assign(AssignStatement::Let { lhs, rhs }) = stmt else {
            panic!("expected Let statement");
        };
        assert_eq!(lhs, &Identifier::new(ValueString::new_static("x")).unwrap());
        assert_eq!(rhs, &binary(int("1"), BinOp::Add, int("2")));
    }

    #[test]
    fn assignment_then_expr() {
        let inner = parse("let x = 5; x");
        assert_eq!(inner.statements.len(), 1);
        assert!(matches!(
            &inner.statements[0],
            Statement::Assign(AssignStatement::Let { .. })
        ));
        assert_eq!(inner.expr, Some(variable("x")));
    }

    #[test]
    fn error_d20_assignment_is_invalid_identifier() {
        assert!(parse_err("d20 = 5").is_invalid_identifier());
    }
}
