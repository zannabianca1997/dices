use dices_values::{Injectable, Value, injectable, injected::call::InjectedContext};

/// module rng
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Rng {
    seed: Seed,
}

impl Rng {
    pub const fn new() -> Self {
        Self { seed: Seed }
    }
}

/// function seed
#[injectable]
fn Seed(#[cx] cx: &mut (impl InjectedContext + ?Sized), args: &[Value]) {
    cx.seed(args);
}
