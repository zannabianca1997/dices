use std::{error::Error, mem};

use dices_man::{Manual, PathComponent};
use dices_values::{
    Injectable, Value, injectable, injected::call::InjectedContext, string::ValueString,
};
use itertools::Itertools;

/// Bindings to the repl
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Repl {
    quit: Quit,
    print: Print,
    print_str: PrintStr,
    print_markdown: PrintMarkdown,
    help: Help,
}

impl Repl {
    pub const fn new() -> Self {
        Self {
            quit: Quit,
            print: Print,
            print_str: PrintStr,
            print_markdown: PrintMarkdown,
            help: Help,
        }
    }
}

/// Print the given values
#[injectable]
fn Print(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    args: &mut [Value],
) -> Result<(), Box<dyn Error>> {
    for value in args {
        cx.print(mem::take(value))?;
    }
    Ok(())
}
/// Print the given string as text
#[injectable]
fn PrintStr(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    value: ValueString,
) -> Result<(), Box<dyn Error>> {
    cx.print_str(value)
}

/// Print the given string as markdown
#[injectable]
fn PrintMarkdown(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    value: ValueString,
) -> Result<(), Box<dyn Error>> {
    cx.print_md(value)
}

/// Stop the calculation and return immediately
#[injectable]
fn Help(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    path: Vec<PathComponent>,
) -> Result<(), Box<dyn Error>> {
    let manual = Manual::new();
    if let Some(page) = manual.fetch(path.clone()) {
        cx.print_manual(&page)
    } else {
        Err(format!("Page {} not found", path.iter().format(".")).into())
    }
}

/// Stop the calculation and return immediately
// TODO: change return to ! when stable
#[injectable]
fn Quit(#[cx] cx: &mut (impl InjectedContext + ?Sized), reason: Value) -> ! {
    cx.abort(reason)
}
