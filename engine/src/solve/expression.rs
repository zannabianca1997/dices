//! Implementations of Solvable on all types of expressions

use closures::VarUseCalcError;
use derive_more::Debug;
use nunny::NonEmpty;

use dices_ast::{
    expression::{
        bin_ops::{BinOp, EvalOrder},
        set::{MemberReceiver, Receiver},
        Expression, ExpressionBinOp, ExpressionCall, ExpressionList, ExpressionMap,
        ExpressionMemberAccess, ExpressionRef, ExpressionScope, ExpressionSet, ExpressionUnOp,
    },
    ident::IdentStr,
    intrisics::InjectedIntr,
    value::{ToListError, ToNumberError, Value, ValueClosure, ValueNull, ValueNumber},
};
pub use intrisics::IntrisicError;
use thiserror::Error;

use crate::{solve::Solvable, DicesRng};

#[derive(Debug, Error)]
pub enum SolveError<InjectedIntrisic: InjectedIntr> {
    #[error("The number of repeats must be a number")]
    RepeatTimesNotANumber(#[source] ToNumberError),
    #[error("The number of repeats must be positive (given {_0})")]
    NegativeRepeats(ValueNumber),
    #[error("The operator {op} needs a number at is right")]
    RHSIsNotANumber {
        op: BinOp,
        #[source]
        source: ToNumberError,
    },
    #[error("The operator {op} needs a number at is left")]
    LHSIsNotANumber {
        op: BinOp,
        #[source]
        source: ToNumberError,
    },
    #[error("The operator {op} needs a list at is right")]
    RHSIsNotAList {
        op: BinOp,
        #[source]
        source: ToListError,
    },
    #[error("The operator {op} needs a list at is left")]
    LHSIsNotAList {
        op: BinOp,
        #[source]
        source: ToListError,
    },
    #[error("Integer overflow")]
    Overflow,
    #[error("The filter operator {op} needs a list of number at his left")]
    FilterNeedNumber {
        op: BinOp,
        #[source]
        source: ToNumberError,
    },
    #[error("The filter operator {} needs a positive number at his right", op)]
    FilterNeedPositive {
        op: BinOp,
        source: <usize as TryFrom<ValueNumber>>::Error,
    },
    #[error("The number of dice faces must be a number")]
    FacesAreNotANumber {
        #[source]
        source: ToNumberError,
    },
    #[error("The number of dice faces must be positive (given {faces})")]
    FacesMustBePositive { faces: ValueNumber },
    #[error("Cannot convert into a number")]
    CannotMakeANumber {
        #[source]
        source: ToNumberError,
    },
    #[error("`*` operator need at least one scalar")]
    MultNeedAScalar,
    #[error("Undefined variable {_0}")]
    InvalidReference(Box<IdentStr>),
    #[error("{_0} is not callable")]
    NotCallable(Value<InjectedIntrisic>),
    #[error("Error during intrisic call")]
    IntrisicError(#[source] Box<RecursionGuard<IntrisicError<InjectedIntrisic>>>),
    #[error("Closures requires {required} params, {given} were instead provided.")]
    WrongNumberOfParams { required: usize, given: usize },
    #[error("The closure failed to calculate what variables needed to be captured")]
    ClosureCannotCalculateCaptures(#[source] VarUseCalcError),
    #[error("{_0} is not indexable")]
    CannotIndex(Value<InjectedIntrisic>),
    #[error("A map can be indexed only by strings, not {_0}")]
    MapIsIndexedByStrings(Value<InjectedIntrisic>),
    #[error("A string can be indexed only by numbers")]
    StringIsIndexedByNumbers(#[source] ToNumberError),
    #[error("A list can be indexed only by numbers")]
    ListIsIndexedByNumbers(#[source] ToNumberError),
    #[error("Index {idx} out of range for string of lenght {len}")]
    StringIndexOutOfRange { idx: ValueNumber, len: usize },
    #[error("Index {idx} out of range for list of lenght {len}")]
    ListIndexOutOfRange { idx: ValueNumber, len: usize },
    #[error("Key not found: \"{_0}\"")]
    MissingKey(dices_ast::value::ValueString),
    #[error("Division by zero")]
    DivisionByZero,
}
impl<InjectedIntrisic: InjectedIntr> From<std::convert::Infallible>
    for SolveError<InjectedIntrisic>
{
    fn from(value: std::convert::Infallible) -> Self {
        match value {}
    }
}

mod recursion_guard;
pub use recursion_guard::RecursionGuard;

impl<InjectedIntrisic> Solvable<InjectedIntrisic> for Expression<InjectedIntrisic>
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        Ok(match self {
            Expression::Const(e) => e.solve(context)?,
            Expression::List(e) => e.solve(context)?,
            Expression::Map(e) => e.solve(context)?,
            Expression::Closure(e) => e.solve(context)?,
            Expression::UnOp(e) => e.solve(context)?,
            Expression::BinOp(e) => e.solve(context)?,
            Expression::Call(e) => e.solve(context)?,
            Expression::Scope(e) => e.solve(context)?,
            Expression::Set(e) => e.solve(context)?,
            Expression::Ref(e) => e.solve(context)?,
            Expression::MemberAccess(e) => e.solve(context)?,
        })
    }
}

impl<InjectedIntrisic> Solvable<InjectedIntrisic> for ExpressionList<InjectedIntrisic>
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        Ok(Value::List(
            self.iter()
                .map(|i| i.solve(context))
                .collect::<Result<_, _>>()?,
        ))
    }
}

impl<InjectedIntrisic> Solvable<InjectedIntrisic> for ExpressionMap<InjectedIntrisic>
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        Ok(Value::Map(
            self.iter()
                .map(|(k, v)| v.solve(context).map(|v| (k.clone(), v)))
                .collect::<Result<_, _>>()?,
        ))
    }
}

mod bin_ops;
mod closures;
mod intrisics;
mod un_ops;

impl<InjectedIntrisic> Solvable<InjectedIntrisic> for ExpressionCall<InjectedIntrisic>
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        let Self { called, params } = self;
        let called = called.solve(context)?;
        let params: Box<_> = params
            .iter()
            .map(|p| p.solve(context))
            .collect::<Result<_, _>>()?;

        match called {
            Value::Intrisic(intrisic) => intrisics::call(intrisic, context, params)
                .map_err(|err| SolveError::IntrisicError(Box::new(RecursionGuard::new(err)))),
            Value::Closure(closure) => {
                let ValueClosure {
                    params: params_names,
                    captures,
                    body,
                } = *closure;
                if params.len() != params_names.len() {
                    return Err(SolveError::WrongNumberOfParams {
                        required: params_names.len(),
                        given: params.len(),
                    });
                }
                context.jailed(|context| {
                    // adding capture vars and params
                    for (name, value) in captures.into_iter().chain(Iterator::zip(
                        params_names.into_vec().into_iter(),
                        params.into_vec(),
                    )) {
                        context.vars_mut().let_(name, value);
                    }
                    // solving in the jailed context
                    body.solve(context)
                })
            }

            _ => Err(SolveError::NotCallable(called)),
        }
    }
}

impl<InjectedIntrisic> Solvable<InjectedIntrisic> for ExpressionMemberAccess<InjectedIntrisic>
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        // first we solve for the accessed value
        let accessed = self.accessed.solve(context)?;
        // then for the index
        let index = self.index.solve(context)?;
        // finally, try to index
        match (accessed, index) {
            (Value::String(s), n) => {
                let n = n
                    .to_number()
                    .map_err(SolveError::StringIsIndexedByNumbers)?;
                let ch = if n >= ValueNumber::ZERO {
                    usize::try_from(n.clone())
                        .ok()
                        .and_then(|n| s.chars().nth(n))
                } else {
                    usize::try_from(n.clone().abs() - 1.into())
                        .ok()
                        .and_then(|n| s.chars().nth_back(n))
                };
                if let Some(ch) = ch {
                    Ok(Value::String(ch.to_string().into()))
                } else {
                    Err(SolveError::StringIndexOutOfRange {
                        idx: n,
                        len: s.chars().count(),
                    })
                }
            }
            (Value::List(l), n) => {
                let n = n
                    .to_number()
                    .map_err(SolveError::StringIsIndexedByNumbers)?;
                let ch = if n >= ValueNumber::ZERO {
                    usize::try_from(n.clone()).ok().and_then(|n| l.get(n))
                } else {
                    usize::try_from(n.clone() + ValueNumber::from(l.len()))
                        .ok()
                        .and_then(|n| l.get(n))
                };
                if let Some(ch) = ch {
                    Ok(ch.clone())
                } else {
                    Err(SolveError::ListIndexOutOfRange {
                        idx: n,
                        len: l.len(),
                    })
                }
            }
            (Value::Map(m), Value::String(s)) => {
                m.get(&s).cloned().ok_or_else(|| SolveError::MissingKey(s))
            }
            (Value::Map(_), idx) => Err(SolveError::MapIsIndexedByStrings(idx)),

            (accessed, _) => Err(SolveError::CannotIndex(accessed)),
        }
    }
}

impl<InjectedIntrisic: InjectedIntr> Solvable<InjectedIntrisic>
    for ExpressionScope<InjectedIntrisic>
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        context.scoped(|context| solve_multiple(self, context))
    }
}

/// Solve multiple expressions, discarding the result of all but the last
pub(crate) fn solve_multiple<R: DicesRng, InjectedIntrisic: InjectedIntr>(
    scope: &NonEmpty<[Expression<InjectedIntrisic>]>,
    context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
) -> Result<Value<InjectedIntrisic>, SolveError<InjectedIntrisic>> {
    let (last, leading) = scope.split_last();
    for expr in leading {
        expr.solve(context)?;
    }
    last.solve(context)
}

impl<InjectedIntrisic> Solvable<InjectedIntrisic> for ExpressionSet<InjectedIntrisic>
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        let value = self.value.solve(context)?;

        match &self.receiver {
            Receiver::Ignore => (),
            Receiver::Set(MemberReceiver { root, indices }) => {
                let indices: Vec<_> = indices
                    .iter()
                    .map(|index| index.solve(context))
                    .collect::<Result<_, _>>()?;
                let mut vars = context.vars_mut();
                let mut destination = vars
                    .get_mut(root)
                    .ok_or_else(|| SolveError::InvalidReference(root.to_owned()))?;
                for index in indices {
                    destination = match (destination, index) {
                        (Value::List(l), n) => {
                            let len = l.len();
                            let n = n
                                .to_number()
                                .map_err(SolveError::StringIsIndexedByNumbers)?;
                            let ch = if n >= ValueNumber::ZERO {
                                usize::try_from(n.clone()).ok().and_then(|n| l.get_mut(n))
                            } else {
                                usize::try_from(n.clone() + ValueNumber::from(l.len()))
                                    .ok()
                                    .and_then(|n| l.get_mut(n))
                            };
                            if let Some(ch) = ch {
                                Ok(ch)
                            } else {
                                Err(SolveError::ListIndexOutOfRange { idx: n, len })
                            }
                        }
                        (Value::Map(m), Value::String(s)) => {
                            Ok(m.entry(s).or_insert(Value::Null(ValueNull)))
                        }
                        (Value::Map(_), idx) => Err(SolveError::MapIsIndexedByStrings(idx)),

                        (accessed, _) => Err(SolveError::CannotIndex(accessed.clone())),
                    }?;
                }
                *destination = value.clone();
            }
            Receiver::Let(v) => context.vars_mut().let_(v.to_owned(), value.clone()),
        }

        Ok(value)
    }
}
impl<InjectedIntrisic> Solvable<InjectedIntrisic> for ExpressionRef
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: DicesRng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic, InjectedIntrisic::Data>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        context
            .vars()
            .get(&self.name)
            .cloned() // todo: is this clone lightweight?
            .ok_or_else(|| SolveError::InvalidReference(self.name.clone()))
    }
}
