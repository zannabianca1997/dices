//! Testing for the examples provided in the manual
#![feature(iter_intersperse)]
#![feature(never_type)]

#[cfg(test)]
mod _test_impl;

include! {env!("MAN_TESTS_RS")}
