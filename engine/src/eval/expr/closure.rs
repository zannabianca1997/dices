use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    sync::Arc,
};

use dices_ast::{expr::closure::ClosureExpr, identifier::Identifier};
use dices_values::{
    Value,
    injected::{
        Injectable, ValueInjected,
        call::{Callable, InjectedContext},
        describable::Describable,
    },
    null::ValueNull,
};
use itertools::Itertools;

use crate::{
    EvalError,
    context::Context,
    var_use::VarUse,
};

pub(crate) fn eval(
    closure_expr: &Arc<ClosureExpr>,
    cx: &mut (impl Context + ?Sized),
) -> Result<Value, EvalError> {
    // Collect all captures
    let captures = captures(closure_expr)
        .into_iter()
        .map(|capture| {
            if let Some(value) = cx.var(&capture) {
                Ok((capture, value.clone()))
            } else {
                Err(EvalError::VariableDoNotExists { name: capture })
            }
        })
        .try_collect()?;

    let closure = InjectedClosure {
        def: Arc::clone(closure_expr),
        captures,
    };

    Ok(ValueInjected::new(closure).into())
}

pub(crate) fn var_use(closure_expr: &ClosureExpr) -> VarUse {
    // Only captures are read
    VarUse {
        reads: captures(closure_expr),
        ..VarUse::none()
    }
}

fn captures(closure_expr: &ClosureExpr) -> BTreeSet<Identifier> {
    super::var_use(&closure_expr.body)
        .reads
        .into_iter()
        .filter(|v| !closure_expr.args.contains(v))
        .collect()
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InjectedClosure {
    def: Arc<ClosureExpr>,
    captures: BTreeMap<Identifier, Value>,
}

impl InjectedClosure {
    fn call_inner(
        &self,
        cx: &mut (impl Context + ?Sized),
        args: &[Value],
    ) -> Result<Value, EvalError> {
        // Variable to set during evaluation
        let vars = self
            .def
            .args
            .iter()
            .enumerate()
            .map(|(pos, arg)| (arg, args.get(pos).unwrap_or(&Value::Null(ValueNull))))
            .chain(self.captures.iter());

        // Set all variables
        //
        // Context is jailed in the call so this is fine
        for (name, value) in vars {
            cx.let_var(name.clone(), value.clone());
        }

        crate::eval::expr::eval(&self.def.body, cx)
    }
}

impl Describable for InjectedClosure {
    fn description(&self) -> impl std::fmt::Display + '_ {
        format!("closure with {} arguments", self.def.args.len())
    }
}

impl Injectable for InjectedClosure {
    fn as_callable(&self) -> Option<&dyn dices_values::injected::call::Callable> {
        Some(self)
    }
}

impl Callable for InjectedClosure {
    fn call(&self, cx: &mut dyn InjectedContext, args: &[Value]) -> Result<Value, Box<dyn Error>> {
        self.call_inner(cx, args)
            .map_err(|err| Box::new(err) as Box<dyn Error>)
    }
}
