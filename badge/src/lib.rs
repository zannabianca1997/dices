//! Core of `dices` behind a C ABI, for the Romhack Camp 2026 badge.
//!
//! This is the REPL engine of the `dices` CLI without any desktop dependency:
//! input is a string, output is plain (uncolored) rendered text, exactly like
//! the default manual printer of the CLI produces.
//!
//! The C API lives in [`ffi`], declared for C consumers in
//! `include/dices_badge.h`. The Rust API ([`BadgeRepl`]) is what the C ABI
//! wraps, and is easier to test.

use std::borrow::Cow;
use std::cell::RefCell;
use std::convert::Infallible;

use dices_engine::{Engine, Evaluator, ui::Ui};
use dices_man::ManPage;
use dices_parser::parse_scope_inner;
use dices_print::{Element, Pretty as _, error::ErrorReport, theme::Style};
use dices_print_cli::render as render_doc;
use dices_std::{Std, StdOptions};
use dices_values::{Value, cast::push_down_if_injected, null::ValueNull, string::ValueString};
use pretty::Arena;
use rand::{Rng, rngs::OsRng};

/// Allocator for the badge firmware.
///
/// Rust's default allocator on ESP-IDF calls `posix_memalign`, which the
/// Tactility app loader does not export to apps, so linking succeeds but
/// relocation fails with "Can't find common posix_memalign". `malloc`/`free`
/// are exported, so allocate through them and keep the raw `malloc` pointer in
/// a header word in front of the returned block to support any alignment.
#[cfg(target_os = "espidf")]
mod esp_alloc {
    use core::alloc::{GlobalAlloc, Layout};
    use core::ffi::c_void;
    use core::ptr;

    unsafe extern "C" {
        fn malloc(size: usize) -> *mut c_void;
        fn free(ptr: *mut c_void);
    }

    /// Size of the header holding the pointer returned by `malloc`.
    const HDR: usize = core::mem::size_of::<usize>();

    pub struct EspHeap;

    unsafe impl GlobalAlloc for EspHeap {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            unsafe {
                let align = layout.align().max(HDR);

                let Some(total) = layout.size().checked_add(align + HDR) else {
                    return ptr::null_mut();
                };

                let raw = malloc(total) as usize;
                if raw == 0 {
                    return ptr::null_mut();
                }

                let addr = (raw + HDR + align - 1) & !(align - 1);
                *((addr - HDR) as *mut usize) = raw;

                addr as *mut u8
            }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
            unsafe {
                if ptr.is_null() {
                    return;
                }

                let raw = *((ptr as usize - HDR) as *const usize);
                free(raw as *mut c_void);
            }
        }
    }
}

#[cfg(target_os = "espidf")]
#[global_allocator]
static ESP_HEAP: esp_alloc::EspHeap = esp_alloc::EspHeap;

/// Write a marker straight to fd 1: no stdio, no buffering, no allocation, so a
/// marker that shows up on the badge console proves execution reached that line.
#[cfg(target_os = "espidf")]
pub(crate) fn raw_mark(s: &str) {
    unsafe extern "C" {
        fn write(fd: i32, buf: *const u8, n: usize) -> isize;
    }

    unsafe {
        let _ = write(1, s.as_ptr(), s.len());
    }
}

#[cfg(not(target_os = "espidf"))]
pub(crate) fn raw_mark(_s: &str) {}

pub mod ffi;
pub mod manual_resource;

use manual_resource::PrerenderedManual;

/// Wrap width chosen to fit the badge display (320 px wide).
pub const DEFAULT_WIDTH: usize = 40;

/// Prompt rendered in front of the manual's code examples.
#[derive(Debug, Clone, Copy)]
pub struct BadgePrompt;

impl dices_print_cli::PromptDisplay for BadgePrompt {
    fn prompt_left(&self) -> Cow<'static, str> {
        ">>> ".into()
    }

    fn prompt_indicator(&self) -> Cow<'static, str> {
        "> ".into()
    }

    fn prompt_multiline_indicator(&self) -> Cow<'static, str> {
        "... ".into()
    }
}

/// How an evaluation ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The input parsed and evaluated.
    Success,
    /// The input did not parse.
    ParseError,
    /// The input parsed but did not evaluate.
    EvalError,
}

impl Outcome {
    /// Whether the input failed to parse or evaluate.
    pub fn is_error(self) -> bool {
        self != Outcome::Success
    }
}

/// The outcome of evaluating an input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// Rendered text: values printed as side effects, then the result of the
    /// last expression, or the rendered error. Without trailing newline.
    pub text: String,
    /// How the evaluation ended.
    pub outcome: Outcome,
}

/// A `dices` REPL without I/O: feed it strings, get rendered text back.
pub struct BadgeRepl {
    engine: Engine,
    width: usize,
    manual_pages: PrerenderedManual,
}

impl Default for BadgeRepl {
    fn default() -> Self {
        Self::new()
    }
}

impl BadgeRepl {
    /// Create a REPL with a random seed and the default wrap width.
    pub fn new() -> Self {
        crate::raw_mark("R:new-entered\n");
        let seed: [u8; 16] = OsRng.r#gen();
        crate::raw_mark("R:seeded\n");

        // TEMP diagnostic: exercise the allocator and Rust's data structures
        // before the engine touches anything, to separate "infrastructure is
        // broken" from "the std's prelude data is broken".
        {
            use std::collections::BTreeMap;

            let mut map: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
            crate::raw_mark("T:map-created\n");

            for i in 0..200u32 {
                map.insert(i, vec![i as u8; 16]);
            }

            crate::raw_mark("T:map-filled\n");

            let sum: u32 = map.values().map(|v| v.len() as u32).sum();
            crate::raw_mark(if sum == 3200 {
                "T:map-sum-ok\n"
            } else {
                "T:map-sum-bad\n"
            });

            let joined: String = map.keys().map(|k| k.to_string()).collect();
            crate::raw_mark(if joined.len() > 100 {
                "T:string-ok\n"
            } else {
                "T:string-bad\n"
            });
        }

        Self::with_seed(seed)
    }

    /// Create a REPL with a fixed seed (for deterministic tests).
    pub fn with_seed(seed: [u8; 16]) -> Self {
        crate::raw_mark("R:constructing-std\n");
        let std = Std::new(StdOptions::sandboxed());
        crate::raw_mark("R:std-constructed\n");
        let engine = Engine::new(seed, std);
        crate::raw_mark("R:engine-constructed\n");
        Self {
            engine,
            width: DEFAULT_WIDTH,
            manual_pages: PrerenderedManual::default(),
        }
    }

    /// Load the prerendered manual (see [`manual_resource`]) so `std.help` can
    /// show pages without the markdown renderer being linked into the app.
    pub fn set_manual(&mut self, blob: &[u8]) -> Result<(), manual_resource::ParseError> {
        self.manual_pages = PrerenderedManual::parse(blob)?;
        Ok(())
    }

    /// Set the wrap width used when rendering output.
    pub fn set_width(&mut self, width: usize) {
        self.width = width;
    }

    /// The current wrap width.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Evaluate an input, like the CLI REPL would:
    ///
    /// - values printed as side effects are rendered first (each followed by a
    ///   newline), like `Ui::print`;
    /// - then the value of the last expression is rendered, unless it is null;
    /// - on parse/eval failure the error is rendered instead;
    /// - a null result renders as nothing (like the CLI).
    pub fn eval(&mut self, input: &str) -> Output {
        #[derive(Debug)]
        enum Failure {
            Parse(dices_parser::ParseCommandError),
            Eval(dices_engine::EvalError),
        }

        let printed = RefCell::new(Vec::<String>::new());
        let ui = CollectUi {
            printed: &printed,
            width: self.width,
            manual_pages: &self.manual_pages,
        };

        let result = parse_scope_inner(&ValueString::new(input.to_owned()))
            .map_err(Failure::Parse)
            .and_then(|scope| self.engine.eval(&scope, ui).map_err(Failure::Eval));

        match result {
            Ok(value) => {
                let mut text = printed.into_inner().join("");
                if !matches!(value, Value::Null(ValueNull)) {
                    // Read an injected value if it's readable, like the CLI does
                    let value = push_down_if_injected(value.clone()).unwrap_or(value);
                    let arena = Arena::new();
                    text.push_str(&render(self.width, &arena, value.with_default_ctx()));
                }
                Output {
                    text,
                    outcome: Outcome::Success,
                }
            }
            Err(failure) => {
                let mut text = printed.into_inner().join("");
                let outcome;
                let arena = Arena::new();
                match &failure {
                    Failure::Parse(err) => {
                        outcome = Outcome::ParseError;
                        text.push_str(&render(self.width, &arena, ErrorReport::new(err)));
                    }
                    Failure::Eval(err) => {
                        outcome = Outcome::EvalError;
                        text.push_str(&render(self.width, &arena, ErrorReport::new(err)));
                    }
                }
                Output { text, outcome }
            }
        }
    }

    /// Forget all variables and re-seed with a fresh random seed.
    pub fn reset(&mut self) {
        self.engine = Engine::new(OsRng.r#gen(), Std::new(StdOptions::sandboxed()));
    }
}

/// Render a printing document to a string, without color, wrapping at `width`.
fn render<'a>(
    width: usize,
    arena: &'a Arena<'a, Element>,
    printing: impl pretty::Pretty<'a, Arena<'a, Element>, Element>,
) -> String {
    let mut buffer = Vec::new();
    render_doc(printing, arena, &mut buffer, width, false, |_| {
        Style::default()
    })
    .expect("rendering to a memory buffer cannot fail");
    String::from_utf8(buffer).expect("rendered text is valid utf-8")
}

/// Ui implementation collecting everything the program prints as side effects.
struct CollectUi<'a> {
    printed: &'a RefCell<Vec<String>>,
    width: usize,
    manual_pages: &'a PrerenderedManual,
}

impl CollectUi<'_> {
    fn push(&self, text: String) {
        self.printed.borrow_mut().push(text);
    }
}

impl Ui for CollectUi<'_> {
    type PrintError = Infallible;

    fn print(&self, value: impl Into<Value>) -> Result<(), Self::PrintError> {
        let arena = Arena::new();
        self.push(render(self.width, &arena, value.into().with_default_ctx()));
        self.push("\n".into());
        Ok(())
    }

    fn print_str<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError> {
        let text = value.as_ref();
        if text.ends_with('\n') {
            self.push(text.to_owned());
        } else {
            self.push(format!("{text}\n"));
        }
        Ok(())
    }

    /// Markdown that the program prints itself is shown as plain text: the
    /// badge does not link a markdown renderer.
    fn print_md<V: AsRef<str> + Into<ValueString>>(
        &self,
        value: V,
    ) -> Result<(), Self::PrintError> {
        self.print_str(value)
    }

    fn manual(&self, item: &ManPage) -> Result<(), Self::PrintError> {
        match self.manual_pages.page(item.path()) {
            Some(text) => {
                self.push(text.to_owned());
                if !text.ends_with('\n') {
                    self.push("\n".into());
                }
            }
            None => self.push("Man page not available on this device.\n".into()),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic() {
        let mut repl = BadgeRepl::with_seed([7; 16]);
        let out = repl.eval("1 + 2");
        assert_eq!(out.outcome, Outcome::Success);
        assert_eq!(out.text, "3");
    }

    #[test]
    fn null_renders_as_nothing() {
        let mut repl = BadgeRepl::with_seed([7; 16]);
        let out = repl.eval("null");
        assert_eq!(out.outcome, Outcome::Success);
        assert_eq!(out.text, "");
    }
}
