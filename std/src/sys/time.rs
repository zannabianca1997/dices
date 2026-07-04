use chrono::{DateTime, Local};
use dices_values::{Injectable, injectable};

/// 5.3.1. Time
///
/// Local time getters.
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

/// 5.3.1.1. Now
///
/// Current time as a ISO8601 format string, with timezone.
///
/// ```dices
/// #>> let now = std.sys.time.now;
/// >>> now()
/// _
/// ```
///
/// Timezone is the local one.
#[injectable]
pub fn Now() -> DateTime<Local> {
    Local::now()
}

/// 5.3.1.2. Timestamp
///
/// Current unix timestamp (seconds since the UNIX epoch)
///
/// ```dices
/// #>> let timestamp = std.sys.time.timestamp;
/// >>> timestamp()
/// _
/// ```
#[injectable]
pub fn Timestamp() -> i64 {
    Now::call().timestamp()
}
