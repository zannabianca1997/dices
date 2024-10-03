//! Implementations of Solvable on all types of expressions

use closures::VarUseCalcError;
use derive_more::{Debug, Display, Error};
use nunny::NonEmpty;

use dices_ast::{
    expression::{
        bin_ops::{BinOp, EvalOrder},
        set::Receiver,
        Expression, ExpressionBinOp, ExpressionCall, ExpressionList, ExpressionMap,
        ExpressionMemberAccess, ExpressionRef, ExpressionScope, ExpressionSet, ExpressionUnOp,
    },
    ident::IdentStr,
    intrisics::InjectedIntr,
    value::{ToListError, ToNumberError, Value, ValueClosure, ValueNumber},
};
pub use intrisics::IntrisicError;

use crate::{solve::Solvable, DicesRng};

#[derive(Debug, Display, Error)]
pub enum SolveError<InjectedIntrisic: InjectedIntr> {
    #[display("The number of repeats must be a number")]
    RepeatTimesNotANumber(#[error(source)] ToNumberError),
    #[display("The number of repeats must be positive (given {_0})")]
    NegativeRepeats(#[error(not(source))] ValueNumber),
    #[display("The operator {op} needs a number at is right")]
    RHSIsNotANumber {
        op: BinOp,
        #[error(source)]
        source: ToNumberError,
    },
    #[display("The operator {op} needs a number at is left")]
    LHSIsNotANumber {
        op: BinOp,
        #[error(source)]
        source: ToNumberError,
    },
    #[display("The operator {op} needs a list at is right")]
    RHSIsNotAList {
        op: BinOp,
        #[error(source)]
        source: ToListError,
    },
    #[display("The operator {op} needs a list at is left")]
    LHSIsNotAList {
        op: BinOp,
        #[error(source)]
        source: ToListError,
    },
    #[display("Integer overflow")]
    Overflow,
    #[display("The filter operator {op} needs a list of number at his left")]
    FilterNeedNumber {
        op: BinOp,
        #[error(source)]
        source: ToNumberError,
    },
    #[display("The filter operator {} needs a positive number at his right", op)]
    FilterNeedPositive {
        op: BinOp,
        source: <usize as TryFrom<ValueNumber>>::Error,
    },
    #[display("The number of dice faces must be a number")]
    FacesAreNotANumber {
        #[error(source)]
        source: ToNumberError,
    },
    #[display("The number of dice faces must be positive (given {faces})")]
    FacesMustBePositive { faces: ValueNumber },
    #[display("Cannot convert into a number")]
    CannotMakeANumber {
        #[error(source)]
        source: ToNumberError,
    },
    #[display("`*` operator need at least one scalar")]
    MultNeedAScalar,
    #[display("Undefined variable {_0}")]
    InvalidReference(#[error(not(source))] Box<IdentStr>),
    #[display("{_0} is not callable")]
    NotCallable(#[error(not(source))] Value<InjectedIntrisic>),
    #[display("Error during intrisic call")]
    IntrisicError(#[error(source)] Box<RecursionGuard<IntrisicError<InjectedIntrisic>>>),
    #[display("Closures requires {required} params, {given} were instead provided.")]
    WrongNumberOfParams { required: usize, given: usize },
    #[display("The closure failed to calculate what variables needed to be captured")]
    ClosureCannotCalculateCaptures(#[error(source)] VarUseCalcError),
    #[display("{_0} is not indexable")]
    CannotIndex(#[error(not(source))] Value<InjectedIntrisic>),
    #[display("A map can be indexed only by strings, not {_0}")]
    MapIsIndexedByStrings(#[error(not(source))] Value<InjectedIntrisic>),
    #[display("A string can be indexed only by numbers")]
    StringIsIndexedByNumbers(#[error(source)] ToNumberError),
    #[display("A list can be indexed only by numbers")]
    ListIsIndexedByNumbers(#[error(source)] ToNumberError),
    #[display("Index {idx} out of range for string of lenght {len}")]
    StringIndexOutOfRange { idx: ValueNumber, len: usize },
    #[display("Index {idx} out of range for list of lenght {len}")]
    ListIndexOutOfRange { idx: ValueNumber, len: usize },
    #[display("Key not found: \"{_0}\"")]
    MissingKey(#[error(not(source))] dices_ast::value::ValueString),
}
impl<InjectedIntrisic: InjectedIntr> From<!> for SolveError<InjectedIntrisic> {
    fn from(value: !) -> Self {
        value
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
        context: &mut crate::Context<R, InjectedIntrisic>,
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
        context: &mut crate::Context<R, InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        Ok(Value::List(
            self.iter().map(|i| i.solve(context)).try_collect()?,
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
        context: &mut crate::Context<R, InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        Ok(Value::Map(
            self.iter()
                .map(|(k, v)| v.solve(context).map(|v| (k.clone(), v)))
                .try_collect()?,
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
        context: &mut crate::Context<R, InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        let Self {
            called: box called,
            params: box params,
        } = self;
        let called = called.solve(context)?;
        let params: Box<_> = params.iter().map(|p| p.solve(context)).try_collect()?;

        match called {
            Value::Intrisic(intrisic) => intrisics::call(intrisic, context, params)
                .map_err(|err| SolveError::IntrisicError(Box::new(RecursionGuard::new(err)))),
            Value::Closure(box ValueClosure {
                params: params_names,
                captures,
                body,
            }) => {
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
                        context.vars_mut().let_(name, value)
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
        context: &mut crate::Context<R, InjectedIntrisic>,
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
                        idx: n.into(),
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
                    Err(SolveError::StringIndexOutOfRange {
                        idx: n.into(),
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
        context: &mut crate::Context<R, InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        context.scoped(|context| solve_multiple(&*self, context))
    }
}

/// Solve multiple expressions, discarding the result of all but the last
pub(crate) fn solve_multiple<R: DicesRng, InjectedIntrisic: InjectedIntr>(
    scope: &NonEmpty<[Expression<InjectedIntrisic>]>,
    context: &mut crate::Context<R, InjectedIntrisic>,
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
        context: &mut crate::Context<R, InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        let value = self.value.solve(context)?;

        match &self.receiver {
            Receiver::Ignore => (),
            Receiver::Set(box v) => {
                *context
                    .vars_mut()
                    .get_mut(v)
                    .ok_or_else(|| SolveError::InvalidReference(v.to_owned()))? = value.clone();
            }
            Receiver::Let(box v) => context.vars_mut().let_(v.to_owned(), value.clone()),
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
        context: &mut crate::Context<R, InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        context
            .vars()
            .get(&self.name)
            .cloned() // todo: is this clone lightweight?
            .ok_or_else(|| SolveError::InvalidReference(self.name.to_owned()))
    }
}
