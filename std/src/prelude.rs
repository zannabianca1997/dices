use dices_values::Injectable;

use crate::convert::{ToBool, ToList, ToNumber, ToString};
use crate::ops::{Join, Sum};
use crate::repl::{Abort, Help, Print};

/// Prelude: commonly-used functions from the standard library
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
