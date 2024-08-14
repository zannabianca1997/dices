use itertools::Itertools;

use super::*;

impl Solvable for ExpressionBinOp {
    type Error = SolveError;

    fn solve<R>(&self, context: &mut crate::Context<R>) -> Result<Value, Self::Error> {
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

fn repeats<R>(
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

fn add<R>(_context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    let [a, b] = ops_to_i64(BinOp::Add, [a, b])?;
    Ok(Value::Number(
        i64::checked_add(a, b).ok_or(SolveError::Overflow)?.into(),
    ))
}

fn mult<R>(_context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    let [a, b] = ops_to_i64(BinOp::Mult, [a, b])?;
    Ok(Value::Number(
        i64::checked_mul(a, b).ok_or(SolveError::Overflow)?.into(),
    ))
}

fn div<R>(_context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    let [a, b] = ops_to_i64(BinOp::Div, [a, b])?;
    Ok(Value::Number(
        i64::checked_div(a, b).ok_or(SolveError::Overflow)?.into(),
    ))
}

fn sub<R>(_context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    let [a, b] = ops_to_i64(BinOp::Sub, [a, b])?;
    Ok(Value::Number(
        i64::checked_sub(a, b).ok_or(SolveError::Overflow)?.into(),
    ))
}

fn rem<R>(_context: &mut crate::Context<R>, a: Value, b: Value) -> Result<Value, SolveError> {
    let [a, b] = ops_to_i64(BinOp::Rem, [a, b])?;
    Ok(Value::Number(
        i64::checked_rem(a, b).ok_or(SolveError::Overflow)?.into(),
    ))
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
