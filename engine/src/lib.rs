//! Engine for the dices programming language
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(iterator_try_reduce)]
#![feature(assert_matches)]
#![feature(unwrap_infallible)]
#![feature(box_patterns)]

pub mod identifier;

pub mod namespace;

pub mod expr;

pub mod value;

pub mod parser;
