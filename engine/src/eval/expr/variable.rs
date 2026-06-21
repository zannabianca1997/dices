use dices_ast::identifier::Identifier;
use dices_values::Value;
use snafu::OptionExt;

use crate::{EvalError, VariableDoNotExistsSnafu, context::Context, var_use::VarUse};

pub(super) fn eval(ident: &Identifier, cx: &mut (impl Context + ?Sized)) -> Result<Value, EvalError> {
    cx.var(ident)
        .cloned()
        .with_context(|| VariableDoNotExistsSnafu {
            name: ident.clone(),
        })
}

pub(crate) fn var_use(identifier: &Identifier) -> VarUse {
    VarUse::read(identifier.clone())
}
