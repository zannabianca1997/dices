#![doc = include_str!("../README.md")]

use dices_values::Injectable;

use convert::Convert;
use serde::{Deserialize, Serialize};
use sys::Sys;

use {ops::Ops, repl::Repl, rng::Rng};

mod convert;
mod ops;
mod repl;
mod rng;
mod sys;

/// Standard library
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Std {
    sys: Sys,
    rng: Rng,
    ops: Ops,
    repl: Repl,
    convert: Convert,
}

impl Std {
    pub const fn new(StdOptions { filesystem }: StdOptions) -> Self {
        Self {
            sys: Sys::new(filesystem),
            rng: Rng::new(),
            ops: Ops::new(),
            repl: Repl::new(),
            convert: Convert::new(),
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
