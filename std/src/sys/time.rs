use chrono::{DateTime, Local};
use dices_values::{Injectable, injectable};

/// Time tools
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Time {
    pub now: Now,
    pub timestamp: Timestamp,
}

impl Time {
    pub const fn new() -> Self {
        Self {
            now: Now,
            timestamp: Timestamp,
        }
    }
}

/// Current time in ISO8601 format
#[injectable]
pub fn Now() -> DateTime<Local> {
    Local::now()
}

/// Current unix timestamp
#[injectable]
pub fn Timestamp() -> i64 {
    Now::call().timestamp()
}
