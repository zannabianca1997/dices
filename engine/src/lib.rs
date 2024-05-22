//! Engine for the dices programming language
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(iterator_try_reduce)]
#![feature(assert_matches)]
#![feature(unwrap_infallible)]

pub mod identifier;

pub mod namespace;

pub mod value;

pub mod intrisics;
