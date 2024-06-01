//! `dice` expression

use std::{collections::HashSet, iter::once, mem, rc::Rc};

use either::Either::{Left, Right};
use itertools::Itertools;
use rand::{Rng, RngCore};
use strum::{EnumDiscriminants, EnumIs, EnumTryAs, IntoStaticStr};
use thiserror::Error;

use crate::{
    identifier::IdentStr,
    namespace::{Missing, Namespace},
    value::{
        div, join, keephigh, keeplow, member_access, mul, neg, rem, removehigh, removelow, sum,
        DString, ToNumberError, Type, Value,
    },
    Callbacks, EvalContext,
};

/// Events that might interrupt an evaluation
#[derive(Debug)]
pub(crate) enum EvalInterrupt {
    Error(EvalError),
    CannotEvalInConst(&'static str),
    Quitted(Box<[Value]>),
}

impl From<EvalError> for EvalInterrupt {
    fn from(value: EvalError) -> Self {
        Self::Error(value)
    }
}

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
    #[error("Number of dice faces must be a number")]
    NaNDiceFaces(#[source] ToNumberError),
    #[error("Negative number of {0}")]
    InvalidNegative(&'static str),
    #[error("Value of type {0} does not support member access")]
    DoesNotSupportMembers(Type),
    #[error("Lists must be indicized with a number")]
    InvalidListIndex(#[source] ToNumberError),
    #[error("Index {0} is out of range for list of lenght {1}")]
    IndexOutOfRange(i64, usize),
    #[error("Member {0} isn't present in the mapping")]
    MissingMapIndex(Rc<str>),
    #[error("Impossible to index a map with a value of type {0}")]
    InvalidMapIndex(Type),
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
    Call {
        fun: Box<Expr>,
        params: Vec<Expr>,
    },

    /// Access of a member
    MemberAccess {
        value: Box<Expr>,
        member: Box<Expr>,
    },

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
    // Filters
    KeepHigh(Box<Expr>, Box<Expr>),
    KeepLow(Box<Expr>, Box<Expr>),
    RemoveHigh(Box<Expr>, Box<Expr>),
    RemoveLow(Box<Expr>, Box<Expr>),
}
impl Expr {
    pub(crate) fn eval<R: Rng, C: Callbacks>(
        &self,
        context: &mut EvalContext<'_, '_, R, C>,
    ) -> Result<Value, EvalInterrupt> {
        let fail_eval = {
            let const_context = context.is_const();
            move |name: &IdentStr| {
                if const_context {
                    EvalInterrupt::CannotEvalInConst("Reference to unknow variable")
                } else {
                    EvalInterrupt::Error(EvalError::UndefinedRef(UndefinedRef(name.to_owned())))
                }
            }
        };
        Ok(match self {
            Expr::Null => Value::Null,
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Number(n) => Value::Number(*n),
            Expr::List(l) => Value::List(l.into_iter().map(|l| l.eval(context)).try_collect()?),
            Expr::String(s) => Value::String(s.clone()),
            Expr::Map(m) => Value::Map(
                m.into_iter()
                    .map(|(n, v)| v.eval(context).map(|v| (n.clone(), v)))
                    .try_collect()?,
            ),
            Expr::Const(val) => val.clone(),
            Expr::Reference(r) => context
                .namespace()
                .get(r)
                .ok_or_else(|| fail_eval(r))?
                .clone(),
            Expr::Function { params, body } => {
                let mut body = Box::into_inner(body.clone());
                body.constant_fold()?; // body is folded before storing

                // capturing context in a series of `let` expressions
                let context: Vec<_> = self
                    .vars()
                    .requires
                    .into_iter()
                    .map(|n| match context.namespace().get(&n) {
                        Some(v) => Ok(Expr::Set {
                            receiver: Receiver::Let(n.into()),
                            value: Box::new(Expr::Const(v.clone())),
                        }),
                        None => Err(fail_eval(n)),
                    })
                    .try_collect()?;
                let mut body = if context.is_empty() {
                    body
                } else {
                    Expr::Scope(context.into_iter().chain(once(body)).collect())
                };

                body.constant_fold()?; // body is folded again so the let expressions can be folded
                Value::Function {
                    params: params.clone().into(),
                    body: body.into(),
                }
            }
            Expr::Call {
                fun,
                params: call_params,
            } => {
                // evaluating the function
                match fun.eval(context)? {
                    Value::Function { params, body } => {
                        if params.len() != call_params.len() {
                            return Err(EvalError::WrongParamNum {
                                expected: params.len(),
                                given: call_params.len(),
                            }
                            .into());
                        }
                        // evaluating params
                        let params = params
                            .iter()
                            .zip(call_params)
                            .map(|(n, p)| p.eval(context).map(|p| (n.clone(), p)))
                            .try_collect()?;
                        /*  creating the namespace with the param values
                        this is not a child of `namespace`, as function cannot see the *current* surrounding context,
                        but only the one captured at the definition */
                        let mut namespace = Namespace::root_with_vars(params);
                        let mut context = match context {
                            EvalContext::Engine {
                                namespace: _,
                                rng,
                                callbacks,
                            } => EvalContext::Engine {
                                namespace: &mut namespace,
                                rng: *rng,
                                callbacks: *callbacks,
                            },
                            EvalContext::Const { namespace: _ } => EvalContext::Const {
                                namespace: &mut namespace,
                            },
                        };
                        // evaluating the body, scoping it accordingly
                        body.eval(&mut context)?
                    }
                    Value::Intrisic(intr) => {
                        // evaluating params
                        let params = call_params
                            .into_iter()
                            .map(|p| p.eval(context))
                            .try_collect()?;
                        /* creating the jail namespace
                        this is not a child of `namespace`, as function cannot see the *current* surrounding context,
                        but only the one captured at the definition
                        Intrisics might be more powerful than this, giving them accesso to the current namespace.
                        That would impede the static examination of the variable access, and context capture */
                        let mut namespace = Namespace::root();
                        let mut context = match context {
                            EvalContext::Engine {
                                namespace: _,
                                rng,
                                callbacks,
                            } => EvalContext::Engine {
                                namespace: &mut namespace,
                                rng: *rng,
                                callbacks: *callbacks,
                            },
                            EvalContext::Const { namespace: _ } => EvalContext::Const {
                                namespace: &mut namespace,
                            },
                        };

                        intr.call(params, &mut context)?
                    }
                    not_callable => return Err(EvalError::NotCallable(not_callable.type_()).into()),
                }
            }
            Expr::Set { receiver, value } => {
                let value = value.eval(context)?;
                receiver
                    .set(context.namespace(), &value)
                    .map_err(|UndefinedRef(r)| fail_eval(&r))?;
                value
            }
            Expr::Scope(exprs) => {
                // scoping
                let mut child_namespace;
                let mut context = match context {
                    EvalContext::Engine {
                        namespace,
                        rng,
                        callbacks,
                    } => {
                        child_namespace = namespace.child();
                        EvalContext::Engine {
                            namespace: &mut child_namespace,
                            rng: *rng,
                            callbacks: *callbacks,
                        }
                    }
                    EvalContext::Const { namespace } => {
                        child_namespace = namespace.child();
                        EvalContext::Const {
                            namespace: &mut child_namespace,
                        }
                    }
                };
                if let Some((last, setup)) = exprs.split_last() {
                    for expr in setup {
                        expr.eval(&mut context)?;
                    }
                    last.eval(&mut context)?
                } else {
                    Value::Null
                }
            }
            Expr::Sum(a) => Value::Number(
                a.iter()
                    .map(|e| e.eval(context).and_then(|a| sum(a).map_err(Into::into)))
                    .try_fold(0i64, |a, b| {
                        b.and_then(|b| a.checked_add(b).ok_or(EvalError::IntegerOverflow.into()))
                    })?,
            ),
            Expr::Neg(a) => neg(a.eval(context)?)?,

            Expr::Mul(a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                mul(a, b)?
            }
            Expr::Div(a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                div(a, b)?
            }
            Expr::Rem(a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                rem(a, b)?
            }

            Expr::Rep(a, n) => {
                let n: u64 = n
                    .eval(context)?
                    .to_number()
                    .map_err(|e| EvalError::from(e))?
                    .try_into()
                    .map_err(|_| EvalError::InvalidNegative("number of repetitions"))?;
                Value::List((0..n).map(|_| a.eval(context)).try_collect()?)
            }
            Expr::Dice(f) => {
                let f: u64 = f
                    .eval(context)?
                    .to_number()
                    .map_err(EvalError::NaNDiceFaces)?
                    .try_into()
                    .map_err(|_| EvalError::InvalidNegative("faces of dice"))?;
                Value::Number(
                    context
                        .rng()
                        .ok_or(EvalInterrupt::CannotEvalInConst("`d` expressions"))?
                        .gen_range(1..=(f as i64)),
                )
            }
            Expr::Join(a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                join(a, b)
            }
            Expr::KeepHigh(a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                keephigh(a, b)?
            }
            Expr::KeepLow(a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                keeplow(a, b)?
            }
            Expr::RemoveHigh(a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                removehigh(a, b)?
            }
            Expr::RemoveLow(a, b) => {
                let a = a.eval(context)?;
                let b = b.eval(context)?;
                removelow(a, b)?
            }
            Expr::MemberAccess { value, member } => {
                let value = value.eval(context)?;
                let member = member.eval(context)?;
                member_access(value, member)?
            }
        })
    }

    pub fn kind(&self) -> ExprKind {
        self.into()
    }

    /// The interaction with the namespace of this expression
    #[inline(always)]
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
            Expr::Mul(a, b)
            | Expr::Div(a, b)
            | Expr::Rem(a, b)
            | Expr::Join(a, b)
            | Expr::KeepHigh(a, b)
            | Expr::KeepLow(a, b)
            | Expr::RemoveHigh(a, b)
            | Expr::RemoveLow(a, b) => VarsDelta::combine(a.vars(), b.vars()),

            // combine is idempotent (`combine(a,a) = a`) so we can collect all the repetitions.
            Expr::Rep(r, n) => VarsDelta::combine(n.vars(), r.vars()),

            Expr::Dice(f) => f.vars(),
            Expr::MemberAccess { value, member } => VarsDelta::combine(value.vars(), member.vars()),
        }
    }

    /// Constant folding
    pub fn constant_fold(&mut self) -> Result<bool, EvalError> {
        // collecting branches to recurse into
        let mut branches = vec![];
        match self {
            Expr::Null
            | Expr::Bool(_)
            | Expr::Number(_)
            | Expr::String(_)
            | Expr::Const(_)
            | Expr::Reference(_) => (),
            Expr::List(l) => branches.extend(l),
            Expr::Map(m) => branches.extend(m.iter_mut().map(|(_, v)| v)),
            Expr::Function {
                params: _,
                box body,
            } => branches.push(body),
            Expr::Call { box fun, params } => branches.extend(once(fun).chain(params)),
            Expr::Set {
                receiver: _,
                box value,
            } => branches.push(value),
            Expr::Scope(s) => branches.extend(s),
            Expr::Sum(a) => branches.extend(a),
            Expr::Neg(box a) | Expr::Dice(box a) => branches.push(a),
            Expr::Mul(box a, box b)
            | Expr::Div(box a, box b)
            | Expr::Rem(box a, box b)
            | Expr::Rep(box a, box b)
            | Expr::Join(box a, box b)
            | Expr::KeepHigh(box a, box b)
            | Expr::KeepLow(box a, box b)
            | Expr::RemoveHigh(box a, box b)
            | Expr::RemoveLow(box a, box b)
            | Expr::MemberAccess {
                value: box a,
                member: box b,
            } => branches.extend([a, b]),
        }
        // recursively fold the branches
        let mut branches_folded = true;
        for b in branches {
            // this avoid using iterator `all` so it is not short circuiting
            let b_folded = b.constant_fold()?;
            branches_folded = b_folded && branches_folded;
        }

        // Special semplifications

        // Scopes cleanup: aside the last expression, all consts can be eliminated as they have no side effects
        if let Expr::Scope(exprs) = self {
            // if there are some non-tail expressions
            if let Some(last) = exprs.pop() {
                // eliminate all non-tail const expressions
                exprs.retain(|e| !matches!(e, Expr::Const(_)));
                // put the tail back
                exprs.push(last)
            }
            // if a single expression remains, and it does not define anything, it can be inlined
            if exprs.len() == 1 && exprs[0].vars().defines.is_empty() {
                let expr = exprs.pop().unwrap();
                *self = expr
            }
        }
        // Sums flattening and collection of consts
        if let Expr::Sum(terms) = self {
            // flattenening
            let mut consts = 0i64;
            for term in mem::take(terms).into_iter().flat_map(|t| {
                if let Expr::Sum(t) = t {
                    Left(t.into_iter())
                } else {
                    Right(once(t))
                }
            }) {
                match term {
                    Expr::Const(v) => {
                        consts = consts
                            .checked_add(sum(v)?)
                            .ok_or(EvalError::IntegerOverflow)?
                    }
                    Expr::Sum(_) => {
                        unreachable!("Sum of sums should be already flattened in branch folding")
                    }
                    term => terms.push(term), // reinserting terms
                }
            }
            // Adding the const term back
            if terms.is_empty() {
                *self = Expr::Const(Value::Number(consts))
            } else {
                if consts != 0 {
                    terms.push(Expr::Const(Value::Number(consts)))
                }
            }
        }

        // can we try to evaluate this node?
        if matches!(self, Expr::Const(_)) {
            // Skip evaluation step
            return Ok(true);
        }
        if !(
            branches_folded  // do not evaluate if the folding did not reach this deep
            ||  matches!(self, Expr::Scope(_))
            // but evaluate scopes anyway, as they might enclose all variables needed to solve them
        ) {
            // constant folding did not reach this node
            // but no error was caused...
            return Ok(false);
        }
        // Evaluating in a const context.
        let mut namespace = Namespace::root();
        match self.eval::<FakeRng, ConstCallbacks>(&mut EvalContext::Const {
            namespace: &mut namespace,
        }) {
            Ok(v) if namespace.is_empty() => {
                // element is correctly evaluated in a const context, and had no side effect.
                // Substituting it with a constant value
                *self = Expr::Const(v);
            }
            Ok(_) => {
                // element is const, but defined something.
                // It cannot be substituted by something const
            }
            Err(EvalInterrupt::Error(err)) => {
                // element errored out, and the error cannot be resolved at runtime
                return Err(err);
            }
            Err(EvalInterrupt::CannotEvalInConst(_reason)) => {
                // element errored out, but the error can be resolved at runtime.
            }
            Err(err) => {
                unreachable!("Other interrupts must not be emitted under const context, but {err:?} was emitted")
            }
        }
        Ok(matches!(self, Expr::Const(_)))
    }
}

/// A dummy rng to solve type constraints
struct FakeRng(!);
impl RngCore for FakeRng {
    fn next_u32(&mut self) -> u32 {
        self.0
    }

    fn next_u64(&mut self) -> u64 {
        self.0
    }

    fn fill_bytes(&mut self, _dest: &mut [u8]) {
        self.0
    }

    fn try_fill_bytes(&mut self, _dest: &mut [u8]) -> Result<(), rand::Error> {
        self.0
    }
}

/// A dummy callback handler
struct ConstCallbacks(!);
impl Callbacks for ConstCallbacks {
    const PRINT_AVAIL: bool = false;

    fn print(&mut self, _value: Value) {
        self.0
    }

    const HELP_AVAIL: bool = false;

    fn help(&mut self, _text: &str) {
        self.0
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
