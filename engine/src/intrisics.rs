//! Basic functions of the interpreter

use rand::Rng;

use crate::{
    namespace::Namespace,
    value::{Expr, ExprPiece, Value},
};

pub fn sum(namespace: &mut Namespace, rng: &mut impl Rng, params: &[Expr]) -> Result<Value, !> {
    let mut total = 0;
    for expr in params {
        let value = expr.eval(namespace, rng)?;
        total += match value {
            Value::Bool(b) => match b {
                true => 1,
                false => 0,
            },
            Value::Number(n) => n,
            // flatten lists of numbers
            Value::List(l) => l
                .into_iter()
                .map::<Result<_, !>, _>(|val| match val.to_number() {
                    Ok(n) => Ok(n),
                    Err(_) => panic!("Sum is valid only on numbers or list of numbers"),
                })
                .try_fold(0, |sum, b| b.map(|b| sum + b))?,

            _ => panic!("{value:?} is invalid in a sum"),
        }
    }
    Ok(Value::Number(total))
}

pub fn join(namespace: &mut Namespace, rng: &mut impl Rng, params: &[Expr]) -> Result<Value, !> {
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

pub fn dice(namespace: &mut Namespace, rng: &mut impl Rng, params: &[Expr]) -> Result<Value, !> {
    let [param] = params else {
        panic!("Dice must have a singular param")
    };
    let faces = param
        .eval(namespace, rng)?
        .to_number()
        .map_err(|_| panic!("Dice need an integer number of faces"))?;
    if faces <= 0 {
        panic!("Invalid number of faces")
    }
    Ok(Value::Number(rng.gen_range(1..=faces)))
}

pub fn set(namespace: &mut Namespace, rng: &mut impl Rng, params: &[Expr]) -> Result<Value, !> {
    let [Expr::Reference(name), value] = params else {
        panic!("Set must be called with a name to set, and a value")
    };
    let value = value.eval(namespace, rng)?;
    let target = namespace
        .get_mut(name)
        .ok_or_else(|| panic!("{name} is undefined"))?;
    *target = value;
    Ok(Value::None)
}

pub fn let_(namespace: &mut Namespace, rng: &mut impl Rng, params: &[Expr]) -> Result<Value, !> {
    let (name, value) = match params {
        [Expr::Reference(name), value] => (name, value),
        [Expr::Reference(name)] => (name, &Expr::None),
        _ => panic!("Let must be called with a name to set, and a optional value"),
    };
    let value = value.eval(namespace, rng)?;
    namespace.let_(name.clone(), value);
    Ok(Value::None)
}

pub fn then(namespace: &mut Namespace, rng: &mut impl Rng, params: &[Expr]) -> Result<Value, !> {
    let Some((last, before)) = params.split_last() else {
        return Ok(Value::None);
    };
    for expr in before {
        expr.eval(namespace, rng)?;
    }
    last.eval(namespace, rng)
}

pub fn scope(namespace: &mut Namespace, rng: &mut impl Rng, params: &[Expr]) -> Result<Value, !> {
    let [body] = params else {
        panic!("Scope must contain a single expression")
    };
    let mut child = namespace.child();
    body.eval(&mut child, rng)
}

#[cfg(test)]
mod tests {
    mod sum {
        use std::assert_matches::assert_matches;

        use rand::{rngs::SmallRng, SeedableRng};

        use super::super::sum;
        use crate::{
            namespace::Namespace,
            value::{
                Expr::{List, Number},
                Value,
            },
        };

        #[test]
        fn nums() {
            let mut namespace = Namespace::root();
            let mut rng = SmallRng::seed_from_u64(0);
            assert_matches!(
                sum(&mut namespace, &mut rng, &[Number(3), Number(2)]),
                Ok(Value::Number(5))
            )
        }

        #[test]
        fn vec() {
            let mut namespace = Namespace::root();
            let mut rng = SmallRng::seed_from_u64(0);
            assert_matches!(
                sum(
                    &mut namespace,
                    &mut rng,
                    &[List(vec![Number(5), Number(6)])]
                ),
                Ok(Value::Number(11))
            )
        }
    }
}
