use std::{collections::HashSet, iter::once};

use derive_more::derive::{Display, Error};
use itertools::Itertools;
use rand::Rng;

use dices_ast::{
    expression::{
        bin_ops::{BinOp, EvalOrder},
        set::Receiver,
        un_ops::UnOp,
        Expression, ExpressionClosure,
    },
    ident::IdentStr,
    intrisics::InjectedIntr,
    values::{Value, ValueClosure},
};

use crate::solve::Solvable;

use super::SolveError;

impl<InjectedIntrisic> Solvable<InjectedIntrisic> for ExpressionClosure<InjectedIntrisic>
where
    InjectedIntrisic: InjectedIntr,
{
    type Error = SolveError<InjectedIntrisic>;

    fn solve<R: Rng>(
        &self,
        context: &mut crate::Context<R, InjectedIntrisic>,
    ) -> Result<Value<InjectedIntrisic>, Self::Error> {
        // pull captures from the context
        let captures = captures(self)
            .map_err(SolveError::ClosureCannotCalculateCaptures)?
            .into_iter()
            .map(|name| {
                context
                    .vars()
                    .get(name)
                    .map(|v| (name.to_owned(), v.clone()))
                    .ok_or_else(|| SolveError::InvalidReference(name.to_owned()))
            })
            .try_collect()?;
        Ok(Value::Closure(Box::new(ValueClosure {
            params: self.params.clone(),
            captures,
            body: (*self.body).clone(),
        })))
    }
}

#[derive(Debug, Clone, Display, Error)]
pub enum VarUseCalcError {
    #[display("The variable(s) `{}` are declared only in some paths", vars.into_iter().format("`, `"))]
    ConditionalLet { vars: HashSet<Box<IdentStr>> },
    #[display("Cannot calculate the variables captured in the closure")]
    CalculateCaptures(Box<VarUseCalcError>),
}

/// This struct contains the interactions that an expression has with all the variables
#[derive(Debug, Clone)]
struct VarUse<'e> {
    /// Variables this expression read the value of
    reads: HashSet<&'e IdentStr>,
    /// Variables this expression set to a value
    sets: HashSet<&'e IdentStr>,
    /// Variables this expression creates/shadows
    lets: HashSet<&'e IdentStr>,
}

impl<'e> VarUse<'e> {
    /// Calculate the use of an expression
    fn of<InjectedIntrisic>(
        expr: &'e Expression<InjectedIntrisic>,
    ) -> Result<Self, VarUseCalcError> {
        fn maybe_concat<'e>(
            a: Result<VarUse<'e>, VarUseCalcError>,
            b: Result<VarUse<'e>, VarUseCalcError>,
        ) -> Result<VarUse<'e>, VarUseCalcError> {
            match (a, b) {
                (Ok(a), Ok(b)) => Ok(a.concat(b)),
                // merge the two problems if compatibles
                (
                    Err(VarUseCalcError::ConditionalLet { vars: vars_a }),
                    Err(VarUseCalcError::ConditionalLet { vars: vars_b }),
                ) => Err(VarUseCalcError::ConditionalLet {
                    vars: vars_a.into_iter().chain(vars_b).collect(),
                }),
                // return one of the two problems
                (Err(err), _) | (_, Err(err)) => Err(err),
            }
        }

        Ok(match expr {
            // const expression do not interact with the variables
            Expression::Const(_) => Self::none(),

            Expression::List(l) => l
                .iter()
                .map(VarUse::of)
                .tree_reduce(maybe_concat)
                .transpose()?
                .unwrap_or_else(VarUse::none),
            Expression::Map(m) => m
                .iter()
                .map(|(_, e)| VarUse::of(e))
                .tree_reduce(maybe_concat)
                .transpose()?
                .unwrap_or_else(VarUse::none),

            Expression::Closure(c) => VarUse {
                reads: captures(c)
                    .map_err(|err| VarUseCalcError::CalculateCaptures(Box::new(err)))?,
                sets: HashSet::new(),
                lets: HashSet::new(),
            },

            Expression::UnOp(un_op) => match un_op.op {
                UnOp::Plus | UnOp::Neg | UnOp::Dice => Self::of(&un_op.expression)?,
            },
            Expression::BinOp(bin_op) => match bin_op.op.eval_order() {
                Some(EvalOrder::AB) => Self::concat(
                    Self::of(&bin_op.expressions[0])?,
                    Self::of(&bin_op.expressions[1])?,
                ),
                Some(EvalOrder::BA) => Self::concat(
                    Self::of(&bin_op.expressions[1])?,
                    Self::of(&bin_op.expressions[0])?,
                ),
                None => match bin_op.op {
                    BinOp::Repeat => {
                        let body_vars = Self::of(&bin_op.expressions[0])?;
                        if body_vars.lets.is_empty() {
                            let n_vars = Self::of(&bin_op.expressions[1])?;
                            Self::concat(n_vars, body_vars)
                        } else {
                            return Err(VarUseCalcError::ConditionalLet {
                                vars: body_vars.lets.into_iter().map(ToOwned::to_owned).collect(),
                            });
                        }
                    }
                    _ => unreachable!(),
                },
            },

            // first the called, then the params in order
            Expression::Call(c) => once(&*c.called)
                .chain(c.params.iter())
                .map(VarUse::of)
                .tree_reduce(maybe_concat)
                .transpose()?
                .unwrap_or_else(VarUse::none),
            // instruction in order, scoped
            Expression::Scope(s) => s
                .iter()
                .map(VarUse::of)
                .tree_reduce(maybe_concat)
                .transpose()?
                .unwrap_or_else(|| unreachable!("The scope should be non empty"))
                .scoped(),
            Expression::Set(s) => {
                Self::concat(
                    // first, the value is calculated
                    Self::of(&s.value)?,
                    // then, the receiver act
                    Self::receiving(&s.receiver),
                )
            }
            Expression::Ref(s) => Self::reads(&s.name),
            Expression::MemberAccess(ma) => {
                Self::concat(Self::of(&ma.accessed)?, Self::of(&ma.index)?)
            }
        })
    }

    /// Expression that do not interact with the variables
    fn none() -> Self {
        Self {
            reads: HashSet::new(),
            sets: HashSet::new(),
            lets: HashSet::new(),
        }
    }
    /// Expression that read a variable
    fn reads(var: &'e IdentStr) -> Self {
        Self {
            reads: HashSet::from([var]),
            sets: HashSet::new(),
            lets: HashSet::new(),
        }
    }
    /// Expression that set a variable
    fn sets(var: &'e IdentStr) -> Self {
        Self {
            reads: HashSet::new(),
            sets: HashSet::from([var]),
            lets: HashSet::new(),
        }
    }
    /// Expression that let a variable
    fn lets(var: &'e IdentStr) -> Self {
        Self {
            reads: HashSet::new(),
            sets: HashSet::new(),
            lets: HashSet::from([var]),
        }
    }

    /// Result of executing first `self`, then `other`
    fn concat<'e2, 'e3>(self, other: VarUse<'e2>) -> VarUse<'e3>
    where
        'e: 'e3,
        'e2: 'e3,
    {
        VarUse {
            // all the one readed by the first, and the one readed by the second
            // UNLESS the first setted or readed them.
            reads: self
                .reads
                .iter()
                .chain(
                    other
                        .reads
                        .difference(&self.sets)
                        .filter(|v| !self.lets.contains(*v)),
                )
                .copied()
                .collect(),
            // all the one setted by the first, and the one setted by the second
            // UNLESS the first shadows them, capturing the set
            sets: self
                .sets
                .iter()
                .chain(other.sets.difference(&self.lets))
                .copied()
                .collect(),
            // lets cannot be cancelled, and simply union
            lets: self.lets.union(&other.lets).copied().collect(),
        }
    }

    /// Close an expression into a scope
    fn scoped(self) -> Self {
        Self {
            // the lets are created in the scope, and do not escape
            lets: HashSet::new(),
            ..self
        }
    }

    /// Calculate the variable use of a receiver
    fn receiving(receiver: &'e Receiver) -> Self {
        match receiver {
            Receiver::Ignore => Self::none(),
            Receiver::Set(box var) => Self::sets(var),
            Receiver::Let(box var) => Self::lets(var),
        }
    }
}

fn captures<InjectedIntrisic>(
    c: &ExpressionClosure<InjectedIntrisic>,
) -> Result<HashSet<&IdentStr>, VarUseCalcError> {
    let VarUse { mut reads, .. } = VarUse::of(&*c.body)?;
    for e in &*c.params {
        reads.remove(&**e);
    }
    Ok(reads)
}
