use std::mem;

use dices_values::{
    Injectable, Value,
    cast::{CastInjectedError, CastIntoIntError},
    injectable,
    injected::{CallError, ValueInjected, call::InjectedContext},
    list::ValueList,
    utils::{deep_sum, join_all},
};

/// Rng bindings
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Ops {
    join: Join,
    sum: Sum,
    call: Call,
}

impl Ops {
    pub const fn new() -> Self {
        Self {
            join: Join,
            sum: Sum,
            call: Call,
        }
    }
}

/// Join multiple collections
#[injectable]
fn Join(args: &mut [Value]) -> Result<Value, CastInjectedError> {
    join_all(args)
}

/// Sum multiple arguments
#[injectable]
fn Sum(args: &mut [Value]) -> Result<Value, CastIntoIntError> {
    deep_sum(args.iter_mut().map(mem::take)).map(Into::into)
}

/// Call a function from a list of args
#[injectable]
fn Call(
    #[cx] cx: &mut dyn InjectedContext,
    fun: ValueInjected,
    args: ValueList,
) -> Result<Value, CallError> {
    fun.call(cx, args.as_slice())
}
