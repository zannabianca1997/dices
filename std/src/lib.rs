#![doc = include_str!("../README.md")]

use dices_values::{Injectable, injected::ValueInjected};

use sys::Sys;

mod sys;

/// std library
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Std {
    sys: Sys,
}

impl Std {
    const fn new() -> Self {
        Self { sys: Sys::new() }
    }

    pub fn inject() -> ValueInjected {
        static INSTANCE: Std = Std::new();
        ValueInjected::new_static(&INSTANCE)
    }
}
