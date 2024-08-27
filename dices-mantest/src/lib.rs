//! Testing for the examples provided in the manual
#![feature(never_type)]

#[cfg(test)]
mod _test_impl;

include! {env!("MAN_TESTS_RS")}
