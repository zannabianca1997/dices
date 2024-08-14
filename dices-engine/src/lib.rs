#![feature(assert_matches)]
#![feature(never_type)]
#![feature(iterator_try_collect)]
#![feature(box_patterns)]

pub use context::Context;

pub mod context;
pub mod solve;
