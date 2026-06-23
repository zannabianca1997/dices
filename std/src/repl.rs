use dices_values::{Injectable, Value, injectable, injected::call::InjectedContext};

/// Bindings to the repl
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Repl {
    quit: Quit,
}

impl Repl {
    pub const fn new() -> Self {
        Self { quit: Quit }
    }
}

/// Stop the calculation and return immediately
// TODO: change return to ! when stable
#[injectable]
fn Quit(#[cx] cx: &mut (impl InjectedContext + ?Sized), reason: Value) -> ! {
    cx.abort(reason)
}
