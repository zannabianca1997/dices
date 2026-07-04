use dices_values::Injectable;

use crate::convert::{ToBool, ToList, ToNumber, ToString};
use crate::ops::{Join, Sum};
use crate::repl::{Abort, Help, Print};

/// 5.6. Prelude
///
/// Common used functions from the standard library. Function present here will
/// be automatically imported in all new sessions.
///
/// ```dices
/// >>> std.prelude
/// <| .. |>
/// ```
///
/// Differently from `std`, is it possible to override the values imported in
/// this way:
///
/// ```dices
/// >>> string(42)
/// "42"
/// >>> string = std.convert.bool;
/// >>> string(42)
/// true
/// ```
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Prelude {
    pub abort: Abort,
    pub print: Print,
    pub help: Help,
    pub join: Join,
    pub sum: Sum,
    pub number: ToNumber,
    pub bool: ToBool,
    pub string: ToString,
    pub list: ToList,
}

impl Prelude {
    pub const fn new() -> Self {
        Self {
            abort: Abort,
            print: Print,
            help: Help,
            join: Join,
            sum: Sum,
            number: ToNumber,
            bool: ToBool,
            string: ToString,
            list: ToList,
        }
    }
}
