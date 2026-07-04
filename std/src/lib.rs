#![doc = include_str!("../README.md")]

use dices_values::Injectable;

use convert::Convert;
use serde::{Deserialize, Serialize};
use sys::Sys;

use {ops::Ops, prelude::Prelude, repl::Repl, rng::Rng};

pub mod convert;
pub mod ops;
pub mod prelude;
pub mod repl;
pub mod rng;
pub mod sys;

/// 5. Standard library
///
/// This is the `dices` std library, providing access to additional
/// functionality to the environment.
///
/// It is always obtainable through the `std` keyword:
///
/// ```dices
/// >>> std
/// <| .. |>
/// ```
///
/// Some parts of it can be missing if not injected, for example `std.sys` will
/// miss filesystem functionality if deactivated.
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Std {
    pub ops: Ops,
    pub rng: Rng,
    pub sys: Sys,
    pub repl: Repl,
    pub convert: Convert,
    pub prelude: Prelude,
}

impl Std {
    pub const fn new(StdOptions { filesystem }: StdOptions) -> Self {
        Self {
            sys: Sys::new(filesystem),
            rng: Rng::new(),
            ops: Ops::new(),
            repl: Repl::new(),
            convert: Convert::new(),
            prelude: Prelude::new(),
        }
    }
}

/// Standard library configuration options
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StdOptions {
    /// Grant access to the filesystem
    pub filesystem: bool,
}

impl Default for StdOptions {
    fn default() -> Self {
        Self { filesystem: true }
    }
}
