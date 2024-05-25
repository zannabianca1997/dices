//! Engine for the dices programming language
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(iterator_try_reduce)]
#![feature(assert_matches)]
#![feature(unwrap_infallible)]
#![feature(box_patterns)]
#![feature(ascii_char)]

pub mod identifier;

pub mod namespace;

pub mod expr;

pub mod value;

#[cfg(feature = "parse")]
pub mod parser;

#[cfg(feature = "pretty")]
mod display;
