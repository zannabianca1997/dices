//! Value of a variable in `dices`

use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
    rc::Rc,
};

use either::Either::{self, Left, Right};
use rand::Rng;
use strum::{EnumDiscriminants, EnumIs, EnumTryAs, IntoStaticStr};
use thiserror::Error;

use crate::{
    identifier::DIdentifier,
    intrisics::{self, DiceError, JoinError, LetError, ScopeError, SetError, SumError},
    namespace::Namespace,
};

type DString = Rc<str>;

#[derive(Debug, Clone, Error)]
#[error("Undefined reference to {0}")]
pub struct UndefinedRef(pub DIdentifier);

/// A node into an expression
pub trait ExprPiece {
    type SpecializeError;

    /// Change all free variables not present in `free_vars` with the one contained in the namespace
    fn specialize(
        self,
        namespace: &Namespace,
        free_vars: &HashSet<DIdentifier>,
    ) -> Result<Self, Self::SpecializeError>
    where
        Self: Sized;

    type EvalError;
    type Value;

    /// Evaluate this expression
    fn eval(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
    ) -> Result<Self::Value, Self::EvalError>;
}

/// A value that can be called
pub trait CallableValue {
    type Error;

    /// Call this value with the given parameters
    ///
    /// Parameters are still unevaluated, so intrisics can decide the order of evaluation
    fn call(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
        params: &[Expr],
    ) -> Result<Value, Self::Error>;
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
    pub fn to_number(&self) -> Result<i64, ToNumberError> {
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
                    return Err(ToNumberError::ListNotSingular(l.len()));
                }
            }

            _ => return Err(ToNumberError::InvalidType(self.type_())),
        })
    }

    pub fn type_(&self) -> Type {
        Type::from(self)
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ToNumberError {
    #[error("List of length {0} is not a valid number")]
    ListNotSingular(usize),
    #[error("Type {0} is not acceptable as a number")]
    InvalidType(Type),
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
#[derive(Debug, Clone, Error)]
#[error("Error while specializing function body")]
pub struct FunctionEvalError(
    #[from]
    #[source]
    <Expr as ExprPiece>::SpecializeError,
);

impl ExprPiece for Function {
    type SpecializeError = <Expr as ExprPiece>::SpecializeError;
    fn specialize(
        self,
        namespace: &Namespace,
        free_vars: &HashSet<DIdentifier>,
    ) -> Result<Self, Self::SpecializeError> {
        let mut params = free_vars.clone();
        params.extend(self.params.iter().cloned());

        Ok(Function {
            params: self.params,
            body: self.body.specialize(namespace, &params)?,
        })
    }

    type Value = Function;
    type EvalError = FunctionEvalError;
    fn eval(
        &self,
        namespace: &mut Namespace,
        _: &mut impl Rng,
    ) -> Result<Self::Value, Self::EvalError> {
        Ok(Function {
            params: self.params.clone(),
            body: self.body.clone().specialize(
                &*namespace,
                &HashSet::from_iter(self.params.iter().cloned()),
            )?,
        })
    }
}

#[derive(Debug, Clone, Error)]
pub enum CallFunctionError {
    #[error("the function takes {expected} params, {given} provided")]
    WrongParamNum { given: usize, expected: usize },
    #[error(transparent)]
    EvalError(#[from] Box<EvalError>),
}
impl CallableValue for Function {
    type Error = CallFunctionError;
    fn call(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
        params: &[Expr],
    ) -> Result<Value, Self::Error> {
        // check the number of arguments is right
        if params.len() != self.params.len() {
            return Err(CallFunctionError::WrongParamNum {
                given: params.len(),
                expected: self.params.len(),
            });
        }
        // create new namespace with the given parameters, evaluated
        let mut namespace: Namespace = self
            .params
            .iter()
            .zip(params.into_iter().map(|p| p.eval(namespace, rng)))
            .map(|(n, e)| e.map(|e| (n.clone(), e)))
            .try_collect()
            .map_err(Box::new)?;
        // body must be able to be evaluated with the parameters alone
        Ok(self.body.eval(&mut namespace, rng).map_err(Box::new)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A callable value
pub enum Callable {
    /// Intrisic functions
    Intrisic(Intrisic),
    /// Function definition
    Function(Rc<Function>),
}
impl CallableValue for Callable {
    type Error = Box<Either<CallFunctionError, CallIntrisicError>>;
    fn call(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
        params: &[Expr],
    ) -> Result<Value, Self::Error> {
        Ok(match self {
            Callable::Function(func) => func.call(namespace, rng, params).map_err(Left)?,
            Callable::Intrisic(intr) => intr.call(namespace, rng, params).map_err(Right)?,
        })
    }
}
impl ExprPiece for Callable {
    type Value = Callable;
    type EvalError = Either<<Function as ExprPiece>::EvalError, <Intrisic as ExprPiece>::EvalError>;
    fn eval(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
    ) -> Result<Self::Value, Self::EvalError> {
        Ok(match self {
            Callable::Function(func) => {
                Callable::Function(Rc::new(func.eval(namespace, rng).map_err(Left)?))
            }
            Callable::Intrisic(intr) => {
                Callable::Intrisic(intr.eval(namespace, rng).map_err(Right)?)
            }
        })
    }

    type SpecializeError = <Function as ExprPiece>::SpecializeError;
    fn specialize(
        self,
        namespace: &Namespace,
        free_vars: &HashSet<DIdentifier>,
    ) -> Result<Self, Self::SpecializeError> {
        Ok(match self {
            Callable::Function(func) => Callable::Function(Rc::new(
                Rc::unwrap_or_clone(func).specialize(namespace, free_vars)?,
            )),
            Callable::Intrisic(intr) => {
                Callable::Intrisic(intr.specialize(namespace, free_vars).into_ok())
            }
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
#[derive(Debug, Clone, Error)]
pub enum CallIntrisicError {
    #[error(transparent)]
    SumError(#[from] SumError),
    #[error(transparent)]
    JoinError(#[from] JoinError),
    #[error(transparent)]
    DiceError(#[from] DiceError),
    #[error(transparent)]
    SetError(#[from] SetError),
    #[error(transparent)]
    ThenError(#[from] EvalError),
    #[error(transparent)]
    LetError(#[from] LetError),
    #[error(transparent)]
    ScopeError(#[from] ScopeError),
}
impl CallableValue for Intrisic {
    type Error = CallIntrisicError;
    fn call(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
        params: &[Expr],
    ) -> Result<Value, Self::Error> {
        Ok(match self {
            Intrisic::Sum => intrisics::sum(namespace, rng, params)?,
            Intrisic::Join => intrisics::join(namespace, rng, params)?,
            Intrisic::Dice => intrisics::dice(namespace, rng, params)?,
            Intrisic::Set => intrisics::set(namespace, rng, params)?,
            Intrisic::Then => intrisics::then(namespace, rng, params)?,
            Intrisic::Let => intrisics::let_(namespace, rng, params)?,
            Intrisic::Scope => intrisics::scope(namespace, rng, params)?,
        })
    }
}

impl ExprPiece for Intrisic {
    type SpecializeError = !;
    fn specialize(
        self,
        _: &Namespace,
        _: &HashSet<DIdentifier>,
    ) -> Result<Self, Self::SpecializeError>
    where
        Self: Sized,
    {
        Ok(self)
    }

    type Value = Self;
    type EvalError = !;

    fn eval(&self, _: &mut Namespace, _: &mut impl Rng) -> Result<Self::Value, Self::EvalError> {
        Ok(*self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumDiscriminants, EnumTryAs)]
#[strum_discriminants(name(ExprKind), derive(EnumIs, IntoStaticStr))]
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

#[derive(Debug, Clone, Error)]
#[error("Variable {0} of type {1} is not callable")]
pub struct NotCallable(pub DIdentifier, pub Type);

#[derive(Debug, Clone, Error)]
pub enum EvalError {
    #[error(transparent)]
    CallError(#[from] <Callable as CallableValue>::Error),
    #[error(transparent)]
    CallableEval(#[from] <Callable as ExprPiece>::EvalError),
    #[error(transparent)]
    UndefinedRef(#[from] UndefinedRef),
    #[error(transparent)]
    NotCallable(#[from] NotCallable),
}

#[derive(Debug, Clone, Error)]
pub enum SpecializeError {
    #[error(transparent)]
    UndefinedRef(#[from] UndefinedRef),
    #[error(transparent)]
    NotCallable(#[from] NotCallable),
}

impl ExprPiece for Expr {
    type EvalError = EvalError;
    fn eval(
        &self,
        namespace: &mut Namespace,
        rng: &mut impl Rng,
    ) -> Result<Value, Self::EvalError> {
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

            Expr::Reference(name) => namespace
                .get(name)
                .ok_or_else(|| UndefinedRef(name.clone()))?
                .clone(),
            Expr::Call {
                called,
                inputs: params,
            } => {
                // resolve the eventual name
                let called = match called {
                    Either::Left(callable) => callable,
                    Either::Right(name) => match namespace.get(name) {
                        Some(Value::Callable(callable)) => callable,
                        Some(val) => {
                            return Err(EvalError::NotCallable(NotCallable(
                                name.clone(),
                                val.type_(),
                            )))
                        }
                        None => return Err(UndefinedRef(name.clone()).into()),
                    },
                }
                .clone();
                // resolve the call
                called.call(namespace, rng, params)?
            }
        })
    }

    type SpecializeError = SpecializeError;
    fn specialize(
        self,
        namespace: &Namespace,
        free_vars: &HashSet<DIdentifier>,
    ) -> Result<Expr, Self::SpecializeError> {
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
                } else {
                    namespace
                        .get(&name)
                        .ok_or_else(|| UndefinedRef(name.clone()))?
                        .clone()
                        .into()
                }
            }
            Expr::Call { called, inputs } => {
                let called = match called {
                    Left(callable) => Left(callable.specialize(namespace, free_vars)?),
                    Right(name) => {
                        if free_vars.contains(&name) {
                            Right(name)
                        } else {
                            let value = namespace
                                .get(&name)
                                .ok_or_else(|| UndefinedRef(name.clone()))?;
                            Left(
                                value
                                    .try_as_callable_ref()
                                    .ok_or_else(|| NotCallable(name.clone(), value.type_()))?
                                    .clone(),
                            )
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

impl Expr {
    pub fn kind(&self) -> ExprKind {
        ExprKind::from(self)
    }
}

impl Display for ExprKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <&'static str as Display>::fmt(&self.into(), f)
    }
}
