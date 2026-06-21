use dices_ast::identifier::Identifier;
use dices_values::Value;
use snafu::OptionExt;

use crate::{EvalError, VariableDoNotExistsSnafu, context::Context};

pub(super) fn eval(ident: &Identifier, cx: &mut Context<'_>) -> Result<Value, EvalError> {
    cx.var(ident)
        .cloned()
        .with_context(|| VariableDoNotExistsSnafu {
            name: ident.clone(),
        })
}
