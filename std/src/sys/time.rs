use chrono::{DateTime, Local};
use dices_values::{Injectable, injectable};

/// module time
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Time {
    now: Now,
    timestamp: Timestamp,
}

impl Time {
    pub const fn new() -> Self {
        Self {
            now: Now,
            timestamp: Timestamp,
        }
    }
}

/// function now
#[injectable]
pub fn Now() -> DateTime<Local> {
    Local::now()
}

/// function timestamp
#[injectable]
pub fn Timestamp() -> i64 {
    Now::call().timestamp()
}
