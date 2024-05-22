//! Basic functions of the interpreter

use rand::Rng;
use thiserror::Error;

use crate::{
    namespace::Namespace,
    value::{EvalError, Expr, ExprKind, ExprPiece, ToNumberError, UndefinedRef, Value},
};

#[derive(Debug, Clone, Error)]
pub enum SumError {
    #[error("Invalid number in sum")]
    NotANumber(
        #[from]
        #[source]
        ToNumberError,
    ),
    #[error("While evaluating addend")]
    EvalError(
        #[from]
        #[source]
        EvalError,
    ),
}
pub fn sum(
    namespace: &mut Namespace,
    rng: &mut impl Rng,
    params: &[Expr],
) -> Result<Value, SumError> {
    let mut total = 0;
    for expr in params {
        let value = expr.eval(namespace, rng)?;
        total += if let Value::List(l) = value {
            l.into_iter()
                .map(|val| val.to_number())
                .try_fold(0, |sum, b| b.map(|b| sum + b))
        } else {
            value.to_number()
        }?
    }
    Ok(Value::Number(total))
}

#[derive(Debug, Clone, Error)]
#[error("While evaluating lists to join")]
pub struct JoinError(
    #[from]
    #[source]
    EvalError,
);
pub fn join(
    namespace: &mut Namespace,
    rng: &mut impl Rng,
    params: &[Expr],
) -> Result<Value, JoinError> {
    let mut total = vec![];
    for expr in params {
        let mut value = expr.eval(namespace, rng)?;
        // flatten lists
        if let Some(l) = value.try_as_list_mut() {
            total.append(l)
        } else {
            total.push(value)
        }
    }
    Ok(Value::List(total))
}

#[derive(Debug, Clone, Error)]
pub enum DiceError {
    #[error("dice takes one param, {0} provided")]
    WrongParamNum(usize),
    #[error("Error in evaluating number of faces")]
    EvalError(
        #[from]
        #[source]
        EvalError,
    ),
    #[error("Number of faces is not a num")]
    NaNFaces(
        #[from]
        #[source]
        ToNumberError,
    ),
    #[error("Number of faces ({0}) is negative")]
    NegFaces(i64),
}
pub fn dice(
    namespace: &mut Namespace,
    rng: &mut impl Rng,
    params: &[Expr],
) -> Result<Value, DiceError> {
    let [param] = params else {
        return Err(DiceError::WrongParamNum(params.len()));
    };
    let faces = param.eval(namespace, rng)?.to_number()?;
    if faces <= 0 {
        return Err(DiceError::NegFaces(faces));
    }
    Ok(Value::Number(rng.gen_range(1..=faces)))
}

#[derive(Debug, Clone, Error)]
pub enum SetError {
    #[error("set takes two params, {0} provided")]
    WrongParamNum(usize),
    #[error("set first param must be a reference, it was a {0}")]
    WrongFirstParam(ExprKind),
    #[error("Error in evaluating value set")]
    EvalError(
        #[from]
        #[source]
        EvalError,
    ),
    #[error(transparent)]
    UndefinedRef(#[from] UndefinedRef),
}
pub fn set(
    namespace: &mut Namespace,
    rng: &mut impl Rng,
    params: &[Expr],
) -> Result<Value, SetError> {
    let (name, value) = match params {
        [Expr::Reference(name), value] => Ok((name, value)),
        [p1, _] => Err(SetError::WrongFirstParam(p1.kind())),
        params => Err(SetError::WrongParamNum(params.len())),
    }?;
    let value = value.eval(namespace, rng)?;
    let target = namespace
        .get_mut(name)
        .ok_or_else(|| UndefinedRef(name.clone()))?;
    *target = value;
    Ok(Value::None)
}

#[derive(Debug, Clone, Error)]
pub enum LetError {
    #[error("let takes one or two params, {0} provided")]
    WrongParamNum(usize),
    #[error("let first param must be a reference, it was a {0}")]
    WrongFirstParam(ExprKind),
    #[error("Error in evaluating initial value for let")]
    EvalError(
        #[from]
        #[source]
        EvalError,
    ),
}
pub fn let_(
    namespace: &mut Namespace,
    rng: &mut impl Rng,
    params: &[Expr],
) -> Result<Value, LetError> {
    let (name, value) = match params {
        [Expr::Reference(name), value] => Ok((name, value)),
        [Expr::Reference(name)] => Ok((name, &Expr::None)),
        [p1, _] => Err(LetError::WrongFirstParam(p1.kind())),
        params => Err(LetError::WrongParamNum(params.len())),
    }?;
    let value = value.eval(namespace, rng)?;
    namespace.let_(name.clone(), value);
    Ok(Value::None)
}

pub fn then(
    namespace: &mut Namespace,
    rng: &mut impl Rng,
    params: &[Expr],
) -> Result<Value, EvalError> {
    let Some((last, before)) = params.split_last() else {
        return Ok(Value::None);
    };
    for expr in before {
        expr.eval(namespace, rng)?;
    }
    last.eval(namespace, rng)
}

#[derive(Debug, Clone, Error)]
pub enum ScopeError {
    #[error("let takes one or two params, {0} provided")]
    WrongParamNum(usize),
    #[error(transparent)]
    EvalError(#[from] EvalError),
}
pub fn scope(
    namespace: &mut Namespace,
    rng: &mut impl Rng,
    params: &[Expr],
) -> Result<Value, ScopeError> {
    let [body] = params else {
        return Err(ScopeError::WrongParamNum(params.len()));
    };
    let mut child = namespace.child();
    Ok(body.eval(&mut child, rng)?)
}

#[cfg(test)]
mod tests {

    macro_rules! check_call {
        (
            $name:ident : $intr:ident ( $( $param:expr ),* ) == $res:pat $( if $guard: expr )? $(, for $reps: expr )?
        ) => {
            #[test]
            fn $name() {
                let mut rng = <::rand::rngs::SmallRng as ::rand::SeedableRng>::seed_from_u64(0);
                for _ in 0..(0 $( + $reps )?) {
                    let mut namespace = crate::namespace::Namespace::root();
                    use crate::value::Value::*;
                    ::std::assert_matches::assert_matches!(
                        $intr(&mut namespace, &mut rng, &[$( $param .into() ),*]),
                        $res $( if $guard )?
                    )
                }
            }
        };
    }

    mod sum {
        use super::super::sum;

        check_call!(nums: sum(Number(3), Number(2)) == Ok(Number(5)));
        check_call!(vec: sum(List(vec![Number(5), Number(6)])) == Ok(Number(11)));
        check_call!(vec_and_num: sum(List(vec![Number(5), Number(6)]), Number(4)) == Ok(Number(15)));
    }

    mod join {
        use super::super::join;

        check_call!(lists: join(
            List(vec![
                Number(3),
                Bool(true)
            ]),
            List(vec![
                Number(42),
                Bool(false)
            ])) == Ok(
                List(list))
                if matches!(*list, [
                    Number(3),
                    Bool(true),
                    Number(42),
                    Bool(false)
                ]
            )
        );
        check_call!(list_and_num: join(
            List(vec![
                Number(3),
                Bool(true)
            ]),
            Number(42)
            ) == Ok(
                List(list))
                if matches!(*list, [
                    Number(3),
                    Bool(true),
                    Number(42),
                ]
            )
        );
        check_call!(single_num: join(
            Number(42)
            ) == Ok(
                List(list))
                if matches!(*list, [
                    Number(42),
                ]
            )
        );
    }

    mod dice {
        use super::super::dice;

        check_call!(d6: dice(Number(6)) == Ok(Number(1..=6)), for 100);
    }

    mod set {
        use ::std::assert_matches::assert_matches;

        use ::rand::{rngs::SmallRng, SeedableRng};

        use super::super::set;
        use crate::{
            identifier::DIdentifier,
            namespace::Namespace,
            value::{Expr, Value::*},
        };

        #[test]
        fn d6() {
            let mut rng = SmallRng::seed_from_u64(0);
            let mut namespace = Namespace::root();

            let ident = DIdentifier::new("a").unwrap();

            namespace.let_(ident.clone(), Number(10));

            assert_matches!(
                set(
                    &mut namespace,
                    &mut rng,
                    &[Expr::Reference(ident.clone()), Bool(true).into()]
                ),
                Ok(None)
            );

            assert_matches!(namespace.get(&ident), Some(Bool(true)))
        }
    }
}
