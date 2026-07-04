use std::{error::Error, mem};

use dices_man::{Manual, PathComponent};
use dices_values::{
    Injectable, Value, injectable, injected::call::InjectedContext, string::ValueString,
};
use itertools::Itertools;

/// 5.4. Repl
///
/// Bindings to the repl
#[derive(Debug, Injectable, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Hash)]
pub struct Repl {
    abort: Abort,
    print: Print,
    print_str: PrintStr,
    print_markdown: PrintMarkdown,
    help: Help,
}

impl Repl {
    pub const fn new() -> Self {
        Self {
            abort: Abort,
            print: Print,
            print_str: PrintStr,
            print_markdown: PrintMarkdown,
            help: Help,
        }
    }
}

/// 5.4.1. Print
///
/// Print the given values
#[injectable]
pub fn Print(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    args: &mut [Value],
) -> Result<(), Box<dyn Error>> {
    for value in args {
        cx.print(mem::take(value))?;
    }
    Ok(())
}
/// 5.4.2. PrintStr
///
/// Print the given string as text
#[injectable]
pub fn PrintStr(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    value: ValueString,
) -> Result<(), Box<dyn Error>> {
    cx.print_str(value)
}

/// 5.4.3. PrintMarkdown
///
/// Print the given string as markdown
#[injectable]
pub fn PrintMarkdown(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    value: ValueString,
) -> Result<(), Box<dyn Error>> {
    cx.print_md(value)
}

/// 5.4.4. Abort
///
/// Stop the calculation and return immediately with the given value.
///
/// ```dices
/// #>> let abort = std.repl.abort;
/// >>> abort("calculation stopped")
/// "calculation stopped"
/// ```
#[injectable]
pub fn Abort(#[cx] cx: &mut (impl InjectedContext + ?Sized), reason: Value) -> ! {
    cx.abort(reason)
}

/// 5.4.5. Help
///
/// Search a page of the manual.
///
/// Without parameters it will print the introduction. Use a list with the
/// section to fetch an exact page.
#[injectable]
pub fn Help(
    #[cx] cx: &mut (impl InjectedContext + ?Sized),
    path: Option<Vec<PathComponent>>,
) -> Result<(), Box<dyn Error>> {
    let manual = Manual::new();

    let Some(path) = path else {
        return cx.print_manual(&manual.first());
    };

    if let Some(page) = manual.fetch(path.clone()) {
        cx.print_manual(&page)
    } else {
        Err(format!("Page {} not found", path.iter().format(".")).into())
    }
}
