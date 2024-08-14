use std::mem;

use dices_ast::values::ValueNull;
use itertools::Itertools;
use un_ops::{neg, plus};

use super::*;

impl Solvable for ExpressionBinOp {
    type Error = SolveError;

    fn solve<R: Rng>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
        let ExpressionBinOp {
            op,
            expressions: box [a, b],
        } = self;
        let [a, b] = match op.eval_order() {
            Some(EvalOrder::AB) => {
                let a = a.solve(context)?;
                let b = b.solve(context)?;
                [a, b]
            }
            Some(EvalOrder::BA) => {
                let b = b.solve(context)?;
                let a = a.solve(context)?;
                [a, b]
            }
            None => {
                return Ok((match op {
                    BinOp::Repeat => repeats,
                    _ => unreachable!("The only special order should be `Repeat`"),
                })(context, a, b)?);
            }
        };
        Ok(match op {
            BinOp::Add => add,
            BinOp::Sub => sub,
            BinOp::Join => join,
            BinOp::Repeat => unreachable!("`Repeat` should be handled aside"),
            BinOp::Mult => mult,
            BinOp::Rem => rem,
            BinOp::Div => div,
            BinOp::KeepHigh => keep_high,
            BinOp::KeepLow => keep_low,
            BinOp::RemoveHigh => remove_high,
            BinOp::RemoveLow => remove_low,
        }(context, a, b)?)
    }
}

fn repeats<R: Rng>(
    context: &mut crate::Context<R>,
    a: &Expression,
    n: &Expression,
) -> Result<Value, SolveError> {
    let repeats: i64 = n
        .solve(context)?
        .to_number()
        .map_err(SolveError::RepeatTimesNotANumber)?
        .into();
    let repeats: u64 = repeats
        .try_into()
        .map_err(|err| SolveError::NegativeRepeats(err))?;
    Ok(Value::List(
        (0..repeats).map(|_| a.solve(context)).try_collect()?,
    ))
}

fn ops_to_i64(op: BinOp, [a, b]: [Value; 2]) -> Result<[i64; 2], SolveError> {
    Ok([
        a.to_number()
            .map_err(|source| SolveError::LHSIsNotANumber { op, source })?
            .into(),
        b.to_number()
            .map_err(|source| SolveError::RHSIsNotANumber { op, source })?
            .into(),
    ])
}

pub(super) fn add<R>(
    context: &mut crate::Context<R>,
    a: Value,
    b: Value,
) -> Result<Value, SolveError> {
    let a = plus(context, a)?.to_number().unwrap().into();
    let b = plus(context, b)?.to_number().unwrap().into();
    Ok(Value::Number(
        i64::checked_add(a, b).ok_or(SolveError::Overflow)?.into(),
    ))
}

pub(super) fn mult<R>(
    _context: &mut crate::Context<R>,
    a: Value,
    b: Value,
) -> Result<Value, SolveError> {
    match (a, b) {
        // scalar and scalar
        (
            a @ (Value::Null(_)
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Intrisic(_)
            | Value::Closure(_)),
            b @ (Value::Null(_)
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Intrisic(_)
            | Value::Closure(_)),
        ) => {
            let [a, b] = ops_to_i64(BinOp::Add, [a, b])?;
            Ok(Value::Number(
                i64::checked_add(a, b).ok_or(SolveError::Overflow)?.into(),
            ))
        }
        // scalar and not
        (
            s @ (Value::Null(_)
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Intrisic(_)
            | Value::Closure(_)),
            Value::List(mut l),
        ) => {
            let s: Value = s
                .to_number()
                .map_err(|source| SolveError::LHSIsNotANumber {
                    op: BinOp::Mult,
                    source,
                })?
                .into();

            for el in l.iter_mut() {
                let v = mem::replace(el, ValueNull.into());
                *el = mult(_context, s.clone(), v)?;
            }

            Ok(l.into())
        }
        (
            Value::List(mut l),
            s @ (Value::Null(_)
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Intrisic(_)
            | Value::Closure(_)),
        ) => {
            let s: Value = s
                .to_number()
                .map_err(|source| SolveError::RHSIsNotANumber {
                    op: BinOp::Mult,
                    source,
                })?
                .into();

            for el in l.iter_mut() {
                let v = mem::replace(el, ValueNull.into());
                *el = mult(_context, v, s.clone())?;
            }

            Ok(l.into())
        }
        (
            s @ (Value::Null(_)
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Intrisic(_)
            | Value::Closure(_)),
            Value::Map(mut m),
        ) => {
            let s: Value = s
                .to_number()
                .map_err(|source| SolveError::LHSIsNotANumber {
                    op: BinOp::Mult,
                    source,
                })?
                .into();

            for (_, el) in m.iter_mut() {
                let v = mem::replace(el, ValueNull.into());
                *el = mult(_context, s.clone(), v)?;
            }

            Ok(m.into())
        }
        (
            Value::Map(mut m),
            s @ (Value::Null(_)
            | Value::Bool(_)
            | Value::Number(_)
            | Value::String(_)
            | Value::Intrisic(_)
            | Value::Closure(_)),
        ) => {
            let s: Value = s
                .to_number()
                .map_err(|source| SolveError::RHSIsNotANumber {
                    op: BinOp::Mult,
                    source,
                })?
                .into();

            for (_, el) in m.iter_mut() {
                let v = mem::replace(el, ValueNull.into());
                *el = mult(_context, v, s.clone())?;
            }

            Ok(m.into())
        }

        // double not scalar
        (Value::List(_) | Value::Map(_), Value::List(_) | Value::Map(_)) => {
            Err(SolveError::MultNeedAScalar)
        }
    }
}

fn sub<R>(context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    // delegate to add and unary `-`
    let b = neg(context, b)?;
    add(context, a, b)
}

fn div<R>(context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    match a {
        Value::List(mut l) => {
            for el in l.iter_mut() {
                let a = mem::replace(el, ValueNull.into());
                *el = div(context, a, b.clone())?;
            }
            Ok(l.into())
        }
        Value::Map(mut m) => {
            for (_, el) in m.iter_mut() {
                let a = mem::replace(el, ValueNull.into());
                *el = div(context, a, b.clone())?;
            }
            Ok(m.into())
        }
        _ => {
            let [a, b] = ops_to_i64(BinOp::Div, [a, b])?;
            Ok(Value::Number(
                i64::checked_div(a, b).ok_or(SolveError::Overflow)?.into(),
            ))
        }
    }
}

fn rem<R>(context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    match a {
        Value::List(mut l) => {
            for el in l.iter_mut() {
                let a = mem::replace(el, ValueNull.into());
                *el = rem(context, a, b.clone())?;
            }
            Ok(l.into())
        }
        Value::Map(mut m) => {
            for (_, el) in m.iter_mut() {
                let a = mem::replace(el, ValueNull.into());
                *el = rem(context, a, b.clone())?;
            }
            Ok(m.into())
        }
        _ => {
            let [a, b] = ops_to_i64(BinOp::Rem, [a, b])?;
            Ok(Value::Number(
                i64::checked_rem(a, b).ok_or(SolveError::Overflow)?.into(),
            ))
        }
    }
}

fn join<R>(_context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    let a = a.to_list().map_err(|source| SolveError::LHSIsNotAList {
        op: BinOp::Join,
        source,
    })?;
    let b = b.to_list().map_err(|source| SolveError::RHSIsNotAList {
        op: BinOp::Join,
        source,
    })?;
    Ok(Value::List(Iterator::chain(a.into_iter(), b).collect()))
}

fn keep_high<R>(_context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    const OP: BinOp = BinOp::KeepHigh;

    let a = a
        .to_list()
        .map_err(|source| SolveError::LHSIsNotAList { op: OP, source })?;
    let b: i64 = b
        .to_number()
        .map_err(|source| SolveError::RHSIsNotANumber { op: OP, source })?
        .into();

    let k: usize = b
        .try_into()
        .map_err(|source| SolveError::FilterNeedPositive { op: OP, source })?;

    let a = a
        .into_iter()
        .map(|v| {
            v.to_number()
                .map_err(|source| SolveError::FilterNeedNumber { op: OP, source })
        })
        .process_results(|r| r.k_largest(k).map(Value::from))?
        .collect();
    Ok(Value::List(a))
}

fn keep_low<R>(_context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    const OP: BinOp = BinOp::KeepLow;

    let a = a
        .to_list()
        .map_err(|source| SolveError::LHSIsNotAList { op: OP, source })?;
    let b: i64 = b
        .to_number()
        .map_err(|source| SolveError::RHSIsNotANumber { op: OP, source })?
        .into();

    let k: usize = b
        .try_into()
        .map_err(|source| SolveError::FilterNeedPositive { op: OP, source })?;

    let a = a
        .into_iter()
        .map(|v| {
            v.to_number()
                .map_err(|source| SolveError::FilterNeedNumber { op: OP, source })
        })
        .process_results(|r| r.k_smallest(k).map(Value::from))?
        .collect();
    Ok(Value::List(a))
}

fn remove_high<R>(
    _context: &mut crate::Context<R>,
    a: Value,
    b: Value,
) -> Result<Value, SolveError> {
    const OP: BinOp = BinOp::RemoveHigh;

    let a = a
        .to_list()
        .map_err(|source| SolveError::LHSIsNotAList { op: OP, source })?;
    let b: i64 = b
        .to_number()
        .map_err(|source| SolveError::RHSIsNotANumber { op: OP, source })?
        .into();

    let k: usize = a.len().saturating_sub(
        b.try_into()
            .map_err(|source| SolveError::FilterNeedPositive { op: OP, source })?,
    );

    let a = a
        .into_iter()
        .map(|v| {
            v.to_number()
                .map_err(|source| SolveError::FilterNeedNumber { op: OP, source })
        })
        .process_results(|r| r.k_smallest(k).map(Value::from))?
        .collect();
    Ok(Value::List(a))
}

fn remove_low<R>(
    _context: &mut crate::Context<R>,
    a: Value,
    b: Value,
) -> Result<Value, SolveError> {
    const OP: BinOp = BinOp::RemoveLow;

    let a = a
        .to_list()
        .map_err(|source| SolveError::LHSIsNotAList { op: OP, source })?;
    let b: i64 = b
        .to_number()
        .map_err(|source| SolveError::RHSIsNotANumber { op: OP, source })?
        .into();

    let k: usize = a.len().saturating_sub(
        b.try_into()
            .map_err(|source| SolveError::FilterNeedPositive { op: OP, source })?,
    );

    let a = a
        .into_iter()
        .map(|v| {
            v.to_number()
                .map_err(|source| SolveError::FilterNeedNumber { op: OP, source })
        })
        .process_results(|r| r.k_largest(k).map(Value::from))?
        .collect();
    Ok(Value::List(a))
}
