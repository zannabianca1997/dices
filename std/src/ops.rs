use std::mem;

use dices_values::{
    Injectable, Value,
    cast::{CastInjectedError, CastIntoIntError},
    injectable,
    injected::{CallError, ValueInjected, call::InjectedContext},
    list::ValueList,
    utils::{deep_sum, join_all},
};

/// 5.1. Operations
///
/// This module provides generalized versions of mathematical operations in
/// function form.
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Ops {
    pub join: Join,
    pub sum: Sum,
    pub call: Call,
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

/// 5.1.1. Join
///
/// Function form of the join `~` operator
///
/// ```dices
/// #>> let join = std.ops.join;
/// >>> join([2,3], 2, null)
/// [2,3,2,null]
/// >>> [2,3] ~ 2 ~ null
/// [2,3,2,null]
/// ```
#[injectable]
pub fn Join(args: &mut [Value]) -> Result<Value, CastInjectedError> {
    join_all(args)
}

/// 5.1.2. Sum
///
/// Function form of the sum `+` operator
///
/// ```dices
/// #>> let sum = std.ops.sum;
/// >>> sum([2,3], 2, null)
/// 7
/// >>> [2,3] + 2 + null
/// 7
/// ```
#[injectable]
pub fn Sum(args: &mut [Value]) -> Result<Value, CastIntoIntError> {
    deep_sum(args.iter_mut().map(mem::take)).map(Into::into)
}

/// 5.1.3. Call
///
/// Call a function with a list of arguments
///
/// ```dices
/// #>> let call = std.ops.call;
/// #>> let join = std.ops.join;
/// >>> call(join, ["Hello", " ", "World!"])
/// "Hello World!"
/// >>> join("Hello", " ", "World!")
/// "Hello World!"
/// ```
#[injectable]
pub fn Call(
    #[cx] cx: &mut dyn InjectedContext,
    fun: ValueInjected,
    args: ValueList,
) -> Result<Value, CallError> {
    fun.call(cx, args.as_slice())
}
