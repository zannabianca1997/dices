//! Value of a variable in `dices`

use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    rc::Rc,
};

use either::Either::{self, Left, Right};
use rand::Rng;
use strum::{EnumDiscriminants, EnumIs, EnumTryAs, IntoStaticStr};

use crate::{identifier::DIdentifier, intrisics, namespace::Namespace};

type DString = Rc<str>;

/// A node into an expression
pub trait ExprPiece {
    /// Change all free variables not present in `free_vars` with the one contained in the namespace
    fn specialize(self, namespace: &Namespace, free_vars: &HashSet<DIdentifier>) -> Result<Self, !>
    where
        Self: Sized;

    type Value;

    /// Evaluate this expression
    fn eval(&self, namespace: &mut Namespace, rng: &mut impl Rng) -> Result<Self::Value, !>;
}

/// A value that can be called
pub trait CallableValue {
    /// Call this value with the given parameters
    ///
    /// Parameters are still unevaluated, so intrisics can decide the order of evaluation
    fn call(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
        params: &[Expr],
    ) -> Result<Value, !>;
}

#[derive(Debug, Clone, EnumDiscriminants, EnumTryAs, PartialEq, Eq)]
#[strum_discriminants(name(Type), derive(EnumIs, IntoStaticStr))]
/// Value of a variable in `dices`
pub enum Value {
    // Plain data types
    None,
    Bool(bool),
    Number(i64),
    List(Vec<Self>),
    String(DString),
    Map(HashMap<DString, Self>),
    // A callable
    Callable(Callable),
}
impl Value {
    pub fn to_number(&self) -> Result<i64, !> {
        Ok(match self {
            Value::Bool(b) => match b {
                true => 1,
                false => 0,
            },
            Value::Number(n) => *n,
            Value::List(l) => {
                if let [n] = &**l {
                    n.to_number()?
                } else {
                    panic!(
                        "{} of lenght {} cannot be parsed as number",
                        Type::List,
                        l.len()
                    )
                }
            }

            _ => panic!("{} cannot be converted to number", self.type_()),
        })
    }

    pub fn type_(&self) -> Type {
        Type::from(self)
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(<&'static str>::from(self), f)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
// Function definition
pub struct Function {
    pub params: Rc<[DIdentifier]>,
    pub body: Expr,
}
impl ExprPiece for Function {
    fn specialize(
        self,
        namespace: &Namespace,
        free_vars: &HashSet<DIdentifier>,
    ) -> Result<Self, !> {
        let mut params = free_vars.clone();
        params.extend(self.params.iter().cloned());

        Ok(Function {
            params: self.params,
            body: self.body.specialize(namespace, &params)?,
        })
    }

    type Value = Function;
    fn eval(&self, namespace: &mut Namespace, _: &mut impl Rng) -> Result<Self::Value, !> {
        Ok(Function {
            params: self.params.clone(),
            body: self.body.clone().specialize(
                &*namespace,
                &HashSet::from_iter(self.params.iter().cloned()),
            )?,
        })
    }
}
impl CallableValue for Function {
    fn call(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
        params: &[Expr],
    ) -> Result<Value, !> {
        // check the number of arguments is right
        if params.len() != self.params.len() {
            panic!("Wrong number of params")
        }
        // create new namespace with the given parameters, evaluated
        let mut namespace: Namespace = self
            .params
            .iter()
            .zip(params.into_iter().map(|p| p.eval(namespace, rng)))
            .map(|(n, e)| e.map(|e| (n.clone(), e)))
            .try_collect()?;
        // body must be able to be evaluated with the parameters alone
        self.body.eval(&mut namespace, rng)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A callable value
pub enum Callable {
    Function(Rc<Function>),
    // Intrisic functions
    Intrisic(Intrisic),
}
impl CallableValue for Callable {
    fn call(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
        params: &[Expr],
    ) -> Result<Value, !> {
        match self {
            Callable::Function(func) => func.call(namespace, rng, params),
            Callable::Intrisic(intr) => intr.call(namespace, rng, params),
        }
    }
}
impl ExprPiece for Callable {
    type Value = Callable;
    fn eval(&self, namespace: &mut Namespace, rng: &mut impl Rng) -> Result<Self::Value, !> {
        match self {
            Callable::Function(func) => Ok(Callable::Function(Rc::new(func.eval(namespace, rng)?))),
            Callable::Intrisic(intr) => Ok(Callable::Intrisic(intr.eval(namespace, rng)?)),
        }
    }

    fn specialize(
        self,
        namespace: &Namespace,
        free_vars: &HashSet<DIdentifier>,
    ) -> Result<Self, !> {
        Ok(match self {
            Callable::Function(func) => Callable::Function(Rc::new(
                Rc::unwrap_or_clone(func).specialize(namespace, free_vars)?,
            )),
            Callable::Intrisic(intr) => Callable::Intrisic(intr.specialize(namespace, free_vars)?),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Intrisic functions
pub enum Intrisic {
    /// `+` operator: Sum the results of multiple throws
    Sum,
    /// `,` operator: Concatenate the results of multiple throws
    Join,
    /// `d` operator: Throw a dice
    Dice,
    /// `=` operator: Set a value
    Set,
    /// `let` operator: Create a value in the current scope
    Let,
    /// `;` operator: Execute a throw, discard the result, execute second throw
    Then,
    /// `{}` operator: Create a new scope
    Scope,
}
impl CallableValue for Intrisic {
    fn call(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
        params: &[Expr],
    ) -> Result<Value, !> {
        match self {
            Intrisic::Sum => intrisics::sum(namespace, rng, params),
            Intrisic::Join => intrisics::join(namespace, rng, params),
            Intrisic::Dice => intrisics::dice(namespace, rng, params),
            Intrisic::Set => intrisics::set(namespace, rng, params),
            Intrisic::Then => intrisics::then(namespace, rng, params),
            Intrisic::Let => intrisics::let_(namespace, rng, params),
            Intrisic::Scope => intrisics::scope(namespace, rng, params),
        }
    }
}

impl ExprPiece for Intrisic {
    fn specialize(self, _: &Namespace, _: &HashSet<DIdentifier>) -> Result<Self, !>
    where
        Self: Sized,
    {
        Ok(self)
    }

    type Value = Self;

    fn eval(&self, _: &mut Namespace, _: &mut impl Rng) -> Result<Self::Value, !> {
        Ok(*self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Value of a expression in `dices`
pub enum Expr {
    // Plain data types
    None,
    Bool(bool),
    Number(i64),
    String(DString),
    List(Vec<Self>),
    Map(HashMap<DString, Self>),
    // A callable
    Callable(Callable),

    // Reference to another name
    Reference(DIdentifier),
    // Function call
    Call {
        called: Either<Callable, DIdentifier>,
        inputs: Vec<Self>,
    },
}

impl From<Value> for Expr {
    fn from(value: Value) -> Self {
        match value {
            Value::None => Expr::None,
            Value::Bool(b) => Expr::Bool(b),
            Value::Number(n) => Expr::Number(n),
            Value::List(l) => Expr::List(l.into_iter().map(Into::into).collect()),
            Value::String(s) => Expr::String(s.clone()),
            Value::Map(m) => Expr::Map(m.into_iter().map(|(n, v)| (n, v.into())).collect()),
            Value::Callable(c) => Expr::Callable(c),
        }
    }
}

impl ExprPiece for Expr {
    fn eval(&self, namespace: &mut Namespace, rng: &mut impl Rng) -> Result<Value, !> {
        Ok(match self {
            Expr::None => Value::None,
            Expr::Bool(b) => Value::Bool(*b),
            Expr::Number(n) => Value::Number(*n),
            Expr::String(s) => Value::String(s.clone()),
            Expr::List(l) => Value::List(l.iter().map(|l| l.eval(namespace, rng)).try_collect()?),
            Expr::Map(fields) => Value::Map(
                fields
                    .iter()
                    .map(|(name, content)| {
                        content
                            .eval(namespace, rng)
                            .map(|content| (name.clone().into(), content))
                    })
                    .try_collect()?,
            ),
            Expr::Callable(c) => Value::Callable(c.eval(namespace, rng)?),

            Expr::Reference(name) => {
                if let Some(val) = namespace.get(name) {
                    val.clone()
                } else {
                    panic!("Invalid reference {name}")
                }
            }
            Expr::Call {
                called,
                inputs: params,
            } => {
                // resolve the eventual name
                let called = match called {
                    Either::Left(callable) => callable,
                    Either::Right(name) => match namespace.get(name) {
                        Some(Value::Callable(callable)) => callable,
                        Some(_) => panic!("Variable {name} is not callable"),
                        None => panic!("Invalid reference {name}"),
                    },
                }
                .clone();
                // resolve the call
                called.call(namespace, rng, params)?
            }
        })
    }

    fn specialize(
        self,
        namespace: &Namespace,
        free_vars: &HashSet<DIdentifier>,
    ) -> Result<Expr, !> {
        Ok(match self {
            Expr::None | Expr::Bool(_) | Expr::Number(_) | Expr::String(_) => self,
            Expr::List(l) => Expr::List(
                l.into_iter()
                    .map(|e| e.specialize(namespace, free_vars))
                    .try_collect()?,
            ),
            Expr::Map(m) => Expr::Map(
                m.into_iter()
                    .map(|(name, val)| val.specialize(namespace, free_vars).map(|val| (name, val)))
                    .try_collect()?,
            ),
            Expr::Callable(callable) => Expr::Callable(callable.specialize(namespace, free_vars)?),
            Expr::Reference(name) => {
                if free_vars.contains(&name) {
                    Expr::Reference(name)
                } else if let Some(value) = namespace.get(&name) {
                    value.clone().into()
                } else {
                    panic!("Unrecognized name {name}")
                }
            }
            Expr::Call { called, inputs } => {
                let called = match called {
                    Left(callable) => Left(callable.specialize(namespace, free_vars)?),
                    Right(name) => {
                        if free_vars.contains(&name) {
                            Right(name)
                        } else if let Some(value) = namespace.get(&name) {
                            if let Value::Callable(callable) = value {
                                Left(callable.clone())
                            } else {
                                panic!("{name} is not callable")
                            }
                        } else {
                            panic!("Unrecognized name {name}")
                        }
                    }
                };
                let inputs = inputs
                    .into_iter()
                    .map(|i| i.specialize(namespace, free_vars))
                    .try_collect()?;

                Expr::Call { called, inputs }
            }
        })
    }

    type Value = Value;
}
