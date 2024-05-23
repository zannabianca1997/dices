//! `dice` expression

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

use rand::Rng;
use strum::{EnumDiscriminants, EnumIs, EnumTryAs, IntoStaticStr};
use thiserror::Error;

use crate::{
    identifier::DIdentifier,
    namespace::Namespace,
    value::{DString, Type, Value},
};

#[derive(Debug, Clone, Error)]
pub enum EvalError {
    #[error(transparent)]
    UndefinedRef(#[from] UndefinedRef),
    #[error("Value of type {0} is not callable")]
    NotCallable(Type),
    #[error("Wrong number of params: {expected} were expected, {given} were given")]
    WrongParamNum { expected: usize, given: usize },
}

#[derive(Debug, Clone, Error)]
#[error("Variable `{0}` is undefined")]
pub struct UndefinedRef(pub DIdentifier);

#[derive(Debug, Clone, EnumDiscriminants, EnumTryAs, PartialEq, Eq)]
#[strum_discriminants(name(ExprKind), derive(EnumIs, IntoStaticStr))]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(tag = "kind", content = "expr")
)]
pub enum Expr {
    // Simple literals and value constructors
    /// Null
    Null,
    /// Boolean
    Bool(bool),
    /// Number
    Number(i64),
    /// List
    List(Vec<Self>),
    /// String
    String(DString),
    /// Map
    Map(HashMap<DString, Self>),

    /// Reference to variables
    Reference(DIdentifier),

    /// Definition of a function
    Function {
        params: Rc<[DIdentifier]>,
        body: Rc<[Statement]>,
    },

    /// Calling a function
    Call { fun: Box<Expr>, params: Vec<Expr> },
}
impl Expr {
    pub fn eval(&self, namespace: &Namespace, rng: &mut impl Rng) -> Result<Value, EvalError> {
        Ok(match self {
            Expr::Null => Value::Null,
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Number(n) => Value::Number(*n),
            Expr::List(l) => Value::List(
                l.into_iter()
                    .map(|l| l.eval(namespace, rng))
                    .try_collect()?,
            ),
            Expr::String(s) => Value::String(s.clone()),
            Expr::Map(m) => Value::Map(
                m.into_iter()
                    .map(|(n, v)| v.eval(namespace, rng).map(|v| (n.clone(), v)))
                    .try_collect()?,
            ),
            Expr::Reference(r) => namespace
                .get(r)
                .ok_or_else(|| UndefinedRef(r.clone()))?
                .clone(),
            Expr::Function { params, body } => {
                let context = self
                    .free_vars()
                    .into_iter()
                    .cloned()
                    .map(|n| match namespace.get(&n) {
                        Some(v) => Ok((n, v.clone())),
                        None => Err(UndefinedRef(n)),
                    })
                    .try_collect()?;
                Value::Function {
                    params: params.clone(),
                    context,
                    body: body.clone(),
                }
            }
            Expr::Call {
                fun,
                params: call_params,
            } => {
                // evaluating the function
                match fun.eval(namespace, rng)? {
                    Value::Function {
                        params,
                        mut context,
                        body,
                    } => {
                        if params.len() != call_params.len() {
                            return Err(EvalError::WrongParamNum {
                                expected: params.len(),
                                given: call_params.len(),
                            });
                        }
                        // evaluating params and adding to the context
                        for (n, p) in params.iter().zip(call_params) {
                            let p = p.eval(namespace, rng)?;
                            context.insert(n.clone(), p);
                        }
                        // creating the namespace with the captured context
                        // this is not a child of `namespace`, as function cannot see the *current* surrounding context,
                        // but only the one captured at the definition
                        let mut namespace = Namespace::root_with_vars(context);
                        // evaluating the body, scoping it accordingly
                        Statement::Scope(body).eval(&mut namespace, rng)?
                    }
                    not_callable => return Err(EvalError::NotCallable(not_callable.type_())),
                }
            }
        })
    }

    pub fn kind(&self) -> ExprKind {
        self.into()
    }

    fn free_vars(&self) -> HashSet<&DIdentifier> {
        match self {
            Expr::Null | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) => HashSet::new(),
            Expr::List(l) => l.into_iter().flat_map(|l| l.free_vars()).collect(),
            Expr::Map(m) => m.values().flat_map(|l| l.free_vars()).collect(),
            Expr::Reference(r) => HashSet::from([r]),
            Expr::Call { box fun, params } => params
                .iter()
                .chain(Some(fun))
                .flat_map(|l| l.free_vars())
                .collect(),
            Expr::Function { params, body } => scope_free_vars(body)
                .into_iter()
                .filter(|var| !params.contains(var))
                .collect(),
        }
    }
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(<&'static str>::from(self), f)
    }
}

#[derive(Debug, Clone, EnumDiscriminants, EnumTryAs, PartialEq, Eq)]
#[strum_discriminants(name(StmKind), derive(EnumIs, IntoStaticStr))]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(tag = "kind", content = "stm")
)]
pub enum Statement {
    /// An expression
    Expr(Expr),

    /// Setting a variable
    Set(DIdentifier, Expr),

    /// Creating a variable
    Let(DIdentifier, Option<Expr>),

    /// Block of statements
    Scope(Rc<[Statement]>),
}
impl Statement {
    pub fn eval(&self, namespace: &mut Namespace, rng: &mut impl Rng) -> Result<Value, EvalError> {
        match self {
            Statement::Expr(expr) => expr.eval(namespace, rng),
            Statement::Set(name, value) => {
                namespace
                    .set(name, value.eval(namespace, rng)?)
                    .map_err(|()| UndefinedRef(name.clone()))?;
                Ok(Value::Null)
            }
            Statement::Let(name, value) => {
                namespace.let_(
                    name.clone(),
                    value
                        .as_ref()
                        .map_or(Ok(Value::Null), |v| v.eval(namespace, rng))?,
                );
                Ok(Value::Null)
            }
            Statement::Scope(stmts) => {
                if let Some((last, leading)) = stmts.split_last() {
                    // scoping into a new namespace
                    let mut namespace = namespace.child();
                    // evaluating all statement except the last
                    for stm in leading {
                        stm.eval(&mut namespace, rng)?;
                    }
                    // returning the value evaluated from the last statement
                    last.eval(&mut namespace, rng)
                } else {
                    Ok(Value::Null)
                }
            }
        }
    }

    pub fn kind(&self) -> StmKind {
        self.into()
    }
}
impl Display for StmKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(<&'static str>::from(self), f)
    }
}

fn scope_free_vars(stms: &[Statement]) -> HashSet<&DIdentifier> {
    let mut free_vars = HashSet::new();
    // going backward in the body
    for stm in stms.iter().rev() {
        match stm {
            // expr use the variables inside it
            // set cannot declare vars
            Statement::Expr(e) | Statement::Set(_, e) => free_vars.extend(e.free_vars()),
            // scope cannot declare vars in the parent scope
            Statement::Scope(s) => free_vars.extend(scope_free_vars(s)),
            Statement::Let(v, i) => {
                // v is declared, so we do not depend on it if used after here
                free_vars.remove(v);
                // the init must be calculated before declaring
                if let Some(i) = i {
                    free_vars.extend(i.free_vars())
                }
            }
        }
    }
    free_vars
}
