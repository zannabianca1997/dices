use std::mem;

use dices_values::{
    Injectable, Value, injectable,
    injected::call::{InjectedContext, ManualError},
    string::ValueString,
};

/// Bindings to the repl
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Repl {
    quit: Quit,
    print: Print,
    help: Help,
}

impl Repl {
    pub const fn new() -> Self {
        Self {
            quit: Quit,
            print: Print,
            help: Help,
        }
    }
}

/// Print the given values
// TODO: change return to ! when stable
#[injectable]
fn Print(#[cx] cx: &mut (impl InjectedContext + ?Sized), args: &mut [Value]) {
    for value in args {
        cx.print(mem::take(value));
    }
}

/// Stop the calculation and return immediately
// TODO: change return to ! when stable
#[injectable]
fn Help(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    page: ValueString,
) -> Result<(), ManualError> {
    cx.manual(page)
}

/// Stop the calculation and return immediately
// TODO: change return to ! when stable
#[injectable]
fn Quit(#[cx] cx: &mut (impl InjectedContext + ?Sized), reason: Value) -> ! {
    cx.abort(reason)
}
