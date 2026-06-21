use dices_ast::statement::assign::{AssignStatement, Lhs};
use snafu::OptionExt;

use crate::{EvalError, VariableDoNotExistsSnafu, context::Context};

pub(super) fn eval(stmt: &AssignStatement, cx: &mut Context<'_>) -> Result<(), EvalError> {
    let (AssignStatement::Let { rhs, .. } | AssignStatement::Set { rhs, .. }) = stmt;
    let rhs = crate::eval::expr::eval(rhs, cx)?;

    match stmt {
        AssignStatement::Let { lhs: ident, rhs: _ } => {
            cx.let_var(ident.clone(), rhs);
            Ok(())
        }
        AssignStatement::Set {
            lhs: Lhs::Variable(ident),
            rhs: _,
        } => {
            *cx.var_mut(ident)
                .with_context(|| VariableDoNotExistsSnafu {
                    name: ident.clone(),
                })? = rhs;
            Ok(())
        }
    }
}
