#![doc = include_str!("../README.md")]

use dices_values::{Injectable, injected::ValueInjected};

use crate::time::Time;

mod time {
    use chrono::{DateTime, Local};
    use dices_values::{Injectable, injectable};

    /// module time
    #[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
    pub struct Time {
        pub now: Now,
    }

    /// function now
    #[injectable]
    pub fn Now() -> DateTime<Local> {
        Local::now()
    }
}

/// std library
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Std {
    time: Time,
}

impl Std {
    pub fn new() -> ValueInjected {
        static INSTANCE: Std = Std {
            time: Time { now: time::Now },
        };

        ValueInjected::new_static(&INSTANCE)
    }
}
