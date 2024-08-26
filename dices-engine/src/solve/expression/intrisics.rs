//! Intrisic operations

use derive_more::derive::{Display, Error};
use dices_ast::{
    expression::{Expression, ExpressionCall},
    intrisics::Intrisic,
    values::{ToListError, Value, ValueIntrisic},
};
use rand::Rng;

use crate::solve::Solvable;

use super::SolveError;

#[derive(Debug, Display, Error, Clone)]
pub enum IntrisicError {
    #[display("Wrong number of params given to the intrisic {}: expected {}, given {given}", called.name(), param_num(called))]
    WrongParamNum { called: Intrisic, given: usize },
    #[display("Expression called failed to evaluate")]
    CallFailed(Box<SolveError>),
    #[display("The second parameter of `call` must be a list of parameters")]
    CallParamsNotAList(ToListError),
}

pub(super) fn call<R: Rng>(
    intrisic: ValueIntrisic,
    context: &mut crate::Context<R>,
    params: Box<[Value]>,
) -> Result<Value, IntrisicError> {
    match intrisic.into() {
        Intrisic::Call => {
            let [called, params] = match Box::<[_; 2]>::try_from(params) {
                Ok(box [a, b]) => [a, b],
                Err(box ref s) => {
                    return Err(IntrisicError::WrongParamNum {
                        called: Intrisic::Call,
                        given: s.len(),
                    })
                }
            };

            ExpressionCall {
                called: Box::new(called.into()),
                params: params
                    .to_list()
                    .map_err(IntrisicError::CallParamsNotAList)?
                    .into_iter()
                    .map(Expression::from)
                    .collect(),
            }
            .solve(context)
            .map_err(|err| IntrisicError::CallFailed(Box::new(err)))
        }
        Intrisic::Sum => todo!(),
        Intrisic::Sub => todo!(),
        Intrisic::Neg => todo!(),
        Intrisic::Join => todo!(),
        Intrisic::Mult => todo!(),
        Intrisic::Rem => todo!(),
        Intrisic::Div => todo!(),
        Intrisic::Dice => todo!(),
        Intrisic::ToNumber => todo!(),
        Intrisic::ToList => todo!(),
        Intrisic::KeepHigh => todo!(),
        Intrisic::KeepLow => todo!(),
        Intrisic::RemoveHigh => todo!(),
        Intrisic::RemoveLow => todo!(),
    }
}

fn param_num(intr: &Intrisic) -> usize {
    match intr {
        Intrisic::Call => 2,
        _ => panic!(),
    }
}
