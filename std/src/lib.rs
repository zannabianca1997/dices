#![doc = include_str!("../README.md")]

use dices_values::Injectable;

use serde::{Deserialize, Serialize};
use sys::Sys;

use crate::rng::Rng;

mod rng;
mod sys;

/// Standard library
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Std {
    sys: Sys,
    rng: Rng,
}

impl Std {
    pub const fn new(StdOptions { filesystem }: StdOptions) -> Self {
        Self {
            sys: Sys::new(filesystem),
            rng: Rng::new(),
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
