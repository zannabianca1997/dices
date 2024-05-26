//! `dice` expression

use std::{collections::HashSet, iter::once, mem, rc::Rc};

use itertools::Itertools;
use rand::Rng;
use strum::{EnumDiscriminants, EnumIs, EnumTryAs, IntoStaticStr};
use thiserror::Error;

use crate::{
    identifier::IdentStr,
    namespace::{Missing, Namespace},
    value::{div, join, mul, neg, rem, sum, DString, ToNumberError, Type, Value},
};

#[derive(Debug, Clone, Error)]
pub enum EvalError {
    #[error(transparent)]
    UndefinedRef(#[from] UndefinedRef),
    #[error("Value of type {0} is not callable")]
    NotCallable(Type),
    #[error("Wrong number of params: {expected} were expected, {given} were given")]
    WrongParamNum { expected: usize, given: usize },
    #[error("Integer overflow")]
    IntegerOverflow,
    #[error(transparent)]
    ToNumberError(#[from] ToNumberError),
    #[error("Invalid operands for `{0}`: {1} and {2}")]
    InvalidTypes(&'static str, Type, Type),
    #[error("Invalid operand for `{0}`: {1}")]
    InvalidType(&'static str, Type),
    #[error("Negative number of repetitions")]
    NegativeRepsNumber,
    #[error("Number of dice faces must be a number")]
    NaNDiceFaces(#[source] ToNumberError),
    #[error("Negative number of dice faces")]
    NegativeDiceFaces,
}

#[derive(Debug, Clone, Error)]
#[error("Variable `{0}` is undefined")]
pub struct UndefinedRef(pub Box<IdentStr>);

impl From<Missing<'_>> for UndefinedRef {
    fn from(value: Missing) -> Self {
        Self(value.0.into())
    }
}

#[derive(Debug, Clone, EnumDiscriminants, EnumTryAs, PartialEq, Eq, Default)]
#[strum_discriminants(name(ExprKind), derive(EnumIs, IntoStaticStr, strum::Display))]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(tag = "kind", content = "expr")
)]
pub enum Expr {
    // Simple literals and value constructors
    /// Null
    #[default]
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
    Map(Vec<(DString, Self)>),

    /// Constant value
    Const(Value),

    /// Reference to variables
    Reference(Rc<IdentStr>),

    /// Definition of a function
    Function {
        params: Box<[Rc<IdentStr>]>,
        body: Box<Expr>,
    },

    /// Calling of a function
    Call { fun: Box<Expr>, params: Vec<Expr> },

    /// Setting a value
    Set {
        receiver: Receiver,
        value: Box<Expr>,
    },

    /// Scope
    Scope(Vec<Expr>),

    // --- Expressions ---
    /// Sum of expressions, flattening lists
    Sum(Vec<Expr>),
    /// Negation of expressions, going inside list
    Neg(Box<Expr>),
    /// Multiplication of two expressions.
    /// One can be a list, as long as the second is a number. In that case the first list is multiplied member by member
    Mul(Box<Expr>, Box<Expr>),
    /// Division of two expressions.
    /// One can be a list, as long as the second is a number. In that case the first list is divided/divide by member by member
    Div(Box<Expr>, Box<Expr>),
    /// Remainder of two expressions.
    /// One can be a list, as long as the second is a number. In that case the first list is divided/divide by member by member
    Rem(Box<Expr>, Box<Expr>),

    /// Repetition of an expression
    /// The second value must be a number, and a list is built by repeating the first expressions
    Rep(Box<Expr>, Box<Expr>),

    /// Dice throw
    Dice(Box<Expr>),

    /// List/String/Map concatenation
    Join(Box<Expr>, Box<Expr>),
}
impl Expr {
    pub fn eval(&self, namespace: &mut Namespace, rng: &mut impl Rng) -> Result<Value, EvalError> {
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
            Expr::Const(val) => val.clone(),
            Expr::Reference(r) => namespace
                .get(r)
                .ok_or_else(|| UndefinedRef((&**r).to_owned()))?
                .clone(),
            Expr::Function { params, body } => {
                let context = self
                    .vars()
                    .requires
                    .into_iter()
                    .map(|n| match namespace.get(&n) {
                        Some(v) => Ok((n.into(), v.clone())),
                        None => Err(UndefinedRef(n.to_owned())),
                    })
                    .try_collect()?;
                Value::Function {
                    params: params.clone().into(),
                    context,
                    body: body.clone().into(),
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
                        body.eval(&mut namespace, rng)?
                    }
                    not_callable => return Err(EvalError::NotCallable(not_callable.type_())),
                }
            }
            Expr::Set { receiver, value } => {
                let value = value.eval(namespace, rng)?;
                receiver.set(namespace, &value)?;
                value
            }
            Expr::Scope(exprs) => {
                // scoping
                let mut namespace = namespace.child();
                if let Some((last, setup)) = exprs.split_last() {
                    for expr in setup {
                        expr.eval(&mut namespace, rng)?;
                    }
                    last.eval(&mut namespace, rng)?
                } else {
                    Value::Null
                }
            }
            Expr::Sum(a) => Value::Number(
                a.iter()
                    .map(|e| e.eval(namespace, rng).and_then(sum))
                    .try_fold(0i64, |a, b| {
                        b.and_then(|b| a.checked_add(b).ok_or(EvalError::IntegerOverflow))
                    })?,
            ),
            Expr::Neg(a) => neg(a.eval(namespace, rng)?)?,

            Expr::Mul(a, b) => {
                let a = a.eval(namespace, rng)?;
                let b = b.eval(namespace, rng)?;
                mul(a, b)?
            }
            Expr::Div(a, b) => {
                let a = a.eval(namespace, rng)?;
                let b = b.eval(namespace, rng)?;
                div(a, b)?
            }
            Expr::Rem(a, b) => {
                let a = a.eval(namespace, rng)?;
                let b = b.eval(namespace, rng)?;
                rem(a, b)?
            }

            Expr::Rep(a, n) => {
                let n: u64 = n
                    .eval(namespace, rng)?
                    .to_number()?
                    .try_into()
                    .map_err(|_| EvalError::NegativeRepsNumber)?;
                Value::List((0..n).map(|_| a.eval(namespace, rng)).try_collect()?)
            }
            Expr::Dice(f) => {
                let f: u64 = f
                    .eval(namespace, rng)?
                    .to_number()
                    .map_err(EvalError::NaNDiceFaces)?
                    .try_into()
                    .map_err(|_| EvalError::NegativeDiceFaces)?;
                Value::Number(rng.gen_range(1..=(f as i64)))
            }
            Expr::Join(a, b) => {
                let a = a.eval(namespace, rng)?;
                let b = b.eval(namespace, rng)?;
                join(a, b)
            }
        })
    }

    pub fn kind(&self) -> ExprKind {
        self.into()
    }

    /// The interaction with the namespace of this expression
    fn vars(&self) -> VarsDelta {
        match self {
            Expr::Null | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) | Expr::Const(_) => {
                VarsDelta::none()
            }

            Expr::Reference(var) => VarsDelta::require(var),

            // list and map evaluate the values in the order they appears
            Expr::List(l) => l
                .iter()
                .map(|e| e.vars())
                .tree_reduce(VarsDelta::combine)
                .unwrap_or_default(),
            Expr::Map(m) => m
                .iter()
                .map(|(_, e)| e.vars())
                .tree_reduce(VarsDelta::combine)
                .unwrap_or_default(),

            Expr::Function { params, body } => VarsDelta {
                requires: body
                    .vars()
                    .requires // all variables required by the body
                    .into_iter()
                    .filter(|v| !params.iter().any(|p| &**p == *v)) // but not contained into the parameters
                    .collect(),
                ..Default::default() // this do not define any variable
            },
            Expr::Call { fun, params } => once(&**fun) // function is evaluated first
                .chain(params) // then all the params, in order
                .map(|e| e.vars())
                .tree_reduce(VarsDelta::combine)
                .unwrap_or_default(),
            Expr::Set { receiver, value } => VarsDelta::combine(
                value.vars(),    // first the value is calculated
                receiver.vars(), // then they are moved into the namespace
            ),
            Expr::Scope(exprs) => {
                let VarsDelta {
                    requires,
                    defines: _,
                } = exprs
                    .iter()
                    .map(|e| e.vars())
                    .tree_reduce(VarsDelta::combine)
                    .unwrap_or_default();
                VarsDelta {
                    requires,
                    defines: Default::default(), // blocks do not define anything
                }
            }
            Expr::Sum(a) => a
                .iter()
                .map(|e| e.vars())
                .tree_reduce(VarsDelta::combine)
                .unwrap_or_default(),
            Expr::Neg(a) => a.vars(),
            Expr::Mul(a, b) | Expr::Div(a, b) | Expr::Rem(a, b) | Expr::Join(a, b) => {
                VarsDelta::combine(a.vars(), b.vars())
            }

            // combine is idempotent (`combine(a,a) = a`) so we can collect all the repetitions.
            Expr::Rep(r, n) => VarsDelta::combine(n.vars(), r.vars()),

            Expr::Dice(f) => f.vars(),
        }
    }

    /// Constant folding
    pub fn constant_fold(&mut self) -> Result<(), EvalError> {
        match self {
            Expr::Null => *self = Expr::Const(Value::Null),
            Expr::Bool(b) => *self = Expr::Const(Value::Bool(*b)),
            Expr::Number(n) => *self = Expr::Const(Value::Number(*n)),
            Expr::List(l) => {
                for e in l.iter_mut() {
                    e.constant_fold()?
                }
                if l.iter().all(Self::is_const) {
                    *self = Expr::Const(Value::List(
                        mem::take(l).into_iter().map(Self::unwrap_const).collect(),
                    ));
                }
            }
            Expr::String(s) => *self = Expr::Const(Value::String(s.clone())),
            Expr::Map(m) => {
                for (_, e) in m.iter_mut() {
                    e.constant_fold()?
                }
                if m.iter().all(|(_, e)| e.is_const()) {
                    *self = Expr::Const(Value::Map(
                        mem::take(m)
                            .into_iter()
                            .map(|(k, v)| (k, v.unwrap_const()))
                            .collect(),
                    ))
                }
            }
            Expr::Function { params, body } => {
                body.constant_fold()?;
                // if it does not capture anything, it can be made const
                if body
                    .vars()
                    .requires // all variables required by the body
                    .into_iter()
                    .filter(|v| !params.iter().any(|p| &**p == *v)) // but not contained into the parameters
                    .next()
                    .is_none()
                {
                    *self = Expr::Const(Value::Function {
                        params: mem::take(params).into(),
                        context: Default::default(),
                        body: mem::take(body).into(),
                    })
                }
            }
            Expr::Call { fun, params } => {
                fun.constant_fold()?;
                for p in params.iter_mut() {
                    p.constant_fold()?
                }
                // TODO: constant calls?
            }
            Expr::Set { value, .. } => value.constant_fold()?,
            Expr::Scope(s) => {
                for e in s.iter_mut() {
                    e.constant_fold()?
                }
                // splitting the last
                let Some(last) = s.pop() else {
                    *self = Expr::Const(Value::Null);
                    return Ok(());
                };
                // all const expression, except the last, can be removed from the body
                // as their value would be discarded
                s.retain(|s| !s.is_const());
                // If a single expression remains, and it does not define any variable, it can be inlined
                // in particular if the body resolved to a constant, this propagate the constant upward
                if s.is_empty() && last.vars().defines.is_empty() {
                    *self = last;
                    return Ok(());
                }
                // reinserting the last expression
                s.push(last)
            }
            Expr::Sum(a) => {
                for addend in a.iter_mut() {
                    addend.constant_fold()?
                }
                // collecting all constants in a single one, while doing constant folding
                let const_term = a
                    .extract_if(|addend| addend.is_const())
                    .map(|a| sum(a.unwrap_const()))
                    .try_fold(0i64, |a, b| {
                        a.checked_add(b?).ok_or(EvalError::IntegerOverflow)
                    })?;
                if a.is_empty() {
                    *self = Expr::Const(Value::Number(const_term))
                } else {
                    a.push(Expr::Const(Value::Number(const_term)))
                }
            }
            Expr::Neg(a) => {
                a.constant_fold()?;
                if a.is_const() {
                    *self = Expr::Const(neg(mem::take(a).unwrap_const())?)
                }
            }
            Expr::Mul(a, b) => {
                a.constant_fold()?;
                b.constant_fold()?;
                if a.is_const() && b.is_const() {
                    *self = Expr::Const(mul(
                        mem::take(a).unwrap_const(),
                        mem::take(b).unwrap_const(),
                    )?)
                }
            }
            Expr::Div(a, b) => {
                a.constant_fold()?;
                b.constant_fold()?;
                if a.is_const() && b.is_const() {
                    *self = Expr::Const(div(
                        mem::take(a).unwrap_const(),
                        mem::take(b).unwrap_const(),
                    )?)
                }
            }
            Expr::Rem(a, b) => {
                a.constant_fold()?;
                b.constant_fold()?;
                if a.is_const() && b.is_const() {
                    *self = Expr::Const(rem(
                        mem::take(a).unwrap_const(),
                        mem::take(b).unwrap_const(),
                    )?)
                }
            }
            Expr::Rep(a, b) => {
                a.constant_fold()?;
                b.constant_fold()?;
                if a.is_const() && b.is_const() {
                    *self = Expr::Const(Value::List(vec![
                        mem::take(a).unwrap_const();
                        mem::take(b)
                            .unwrap_const()
                            .to_number()?
                            .try_into()
                            .map_err(|_| {
                                EvalError::NegativeRepsNumber
                            })?
                    ]))
                }
            }
            Expr::Dice(f) => f.constant_fold()?,
            Expr::Join(a, b) => {
                a.constant_fold()?;
                b.constant_fold()?;
                if a.is_const() && b.is_const() {
                    *self = Expr::Const(join(
                        mem::take(a).unwrap_const(),
                        mem::take(b).unwrap_const(),
                    ))
                }
            }

            // nothing to do here
            Expr::Const(_) | Expr::Reference(_) => (),
        }
        Ok(())
    }
    fn is_const(&self) -> bool {
        matches!(self, Expr::Const(_))
    }
    fn unwrap_const(self) -> Value {
        let Expr::Const(v) = self else { unreachable!() };
        v
    }
}

#[derive(Debug, Default, Clone)]
/// Interaction with the namespace of a given expression
struct VarsDelta<'i> {
    /// BEFORE this expression is evaluated, the namespace must contain these variables
    requires: HashSet<&'i IdentStr>,
    /// AFTER this expression is evaluated, these variables will be present in the namespace
    defines: HashSet<&'i IdentStr>,
}

impl<'i> VarsDelta<'i> {
    /// Combine the deltas of two consecutively evaluated expressions
    ///
    /// Associative: `combine(combine(a,b),c) == combine(a,combine(b,c))`
    fn combine(
        Self {
            requires: before_requires,
            defines: before_defines,
        }: Self,
        Self {
            requires: after_requires,
            defines: after_defines,
        }: Self,
    ) -> Self {
        Self {
            requires: after_requires // what is required for evaluating the second expression
                .difference(&before_defines) // but is not defined by the first
                .copied()
                .chain(before_requires) // plus what is required by the first
                .collect(),
            defines: before_defines.union(&after_defines).copied().collect(),
        }
    }

    fn require(var: &IdentStr) -> VarsDelta {
        VarsDelta {
            defines: HashSet::new(),
            requires: HashSet::from([var]),
        }
    }

    fn define(var: &IdentStr) -> VarsDelta {
        VarsDelta {
            defines: HashSet::from([var]),
            requires: HashSet::new(),
        }
    }

    fn none() -> VarsDelta<'static> {
        VarsDelta {
            defines: HashSet::new(),
            requires: HashSet::new(),
        }
    }
}

/// Something that can be set to
#[derive(Debug, Clone, EnumDiscriminants, EnumTryAs, PartialEq, Eq)]
#[strum_discriminants(name(ReceiverKind), derive(EnumIs, IntoStaticStr, strum::Display))]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(tag = "kind", content = "expr")
)]
pub enum Receiver {
    /// Setting a variable
    Set(Rc<IdentStr>),
    /// Creating or shadowing a variable
    Let(Rc<IdentStr>),
    /// Discard the value
    Discard,
}
impl Receiver {
    fn set(&self, namespace: &mut Namespace, value: &Value) -> Result<(), UndefinedRef> {
        match self {
            Receiver::Discard => (),
            Receiver::Set(var) => namespace.set(&*var, value.clone())?,
            Receiver::Let(var) => namespace.let_(var.clone(), value.clone()),
        };
        Ok(())
    }

    /// The interaction with the namespace of this receiver
    fn vars(&self) -> VarsDelta {
        match self {
            Receiver::Set(var) => VarsDelta::require(var),
            Receiver::Let(var) => VarsDelta::define(var),
            Receiver::Discard => VarsDelta::none(),
        }
    }
}
