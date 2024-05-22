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
