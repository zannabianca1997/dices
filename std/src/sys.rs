use dices_values::Injectable;
use time::Time;

mod time;

/// System bindings
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Sys {
    time: Time,
}

impl Sys {
    pub const fn new() -> Self {
        Self { time: Time::new() }
    }
}
