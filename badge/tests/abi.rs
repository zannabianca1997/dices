//! End-to-end tests of the C ABI, driven from Rust.

use std::ffi::{CStr, CString};

use dices_badge::ffi::{
    DICES_STATUS_EVAL_ERROR, DICES_STATUS_INVALID_ARG, DICES_STATUS_OK, DICES_STATUS_PARSE_ERROR,
    DICES_STATUS_TRUNCATED, dices_repl_eval, dices_repl_free, dices_repl_new, dices_repl_reset,
    dices_repl_set_width,
};

struct Repl(*mut dices_badge::BadgeRepl);

impl Repl {
    fn new() -> Self {
        let repl = dices_repl_new();
        assert!(!repl.is_null());
        Self(repl)
    }

    fn eval(&self, input: &str) -> (i32, String) {
        let input = CString::new(input).unwrap();
        let mut buffer = [0u8; 1024];
        let status = unsafe {
            dices_repl_eval(
                self.0,
                input.as_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
            )
        };
        let text = unsafe { CStr::from_ptr(buffer.as_ptr().cast()) }
            .to_string_lossy()
            .into_owned();
        (status, text)
    }
}

impl Drop for Repl {
    fn drop(&mut self) {
        unsafe { dices_repl_free(self.0) }
    }
}

#[test]
fn arithmetic() {
    let repl = Repl::new();
    let (status, text) = repl.eval("1 + 2");
    assert_eq!(status, DICES_STATUS_OK);
    assert_eq!(text, "3");
}

#[test]
fn dice_throw() {
    let repl = Repl::new();
    let (status, text) = repl.eval("d6");
    assert_eq!(status, DICES_STATUS_OK);
    // The manual's `1..=6` is a matcher: an actual throw renders a concrete value
    let roll: i64 = text.parse().expect("d6 should render a number");
    assert!((1..=6).contains(&roll), "d6 rendered {roll}");
}

#[test]
fn variables_persist_across_calls() {
    let repl = Repl::new();

    // `let` returns null, which renders as nothing
    let (status, text) = repl.eval("let x = 40;");
    assert_eq!(status, DICES_STATUS_OK);
    assert_eq!(text, "");

    let (status, text) = repl.eval("x + 2");
    assert_eq!(status, DICES_STATUS_OK);
    assert_eq!(text, "42");
}

#[test]
fn multiline_input() {
    let repl = Repl::new();
    let (status, text) = repl.eval("{\n  let x = 3;\n  x * 7\n}");
    assert_eq!(status, DICES_STATUS_OK);
    assert_eq!(text, "21");
}

#[test]
fn parse_error() {
    let repl = Repl::new();
    let (status, text) = repl.eval("let = ;");
    assert_eq!(status, DICES_STATUS_PARSE_ERROR);
    assert!(!text.is_empty());
}

#[test]
fn eval_error() {
    let repl = Repl::new();
    let (status, text) = repl.eval("this_variable_does_not_exist");
    assert_eq!(status, DICES_STATUS_EVAL_ERROR);
    assert!(!text.is_empty());
}

#[test]
fn reset_forgets_variables() {
    let repl = Repl::new();
    repl.eval("let x = 42;");
    unsafe { dices_repl_reset(repl.0) };
    let (status, _) = repl.eval("x");
    assert_eq!(status, DICES_STATUS_EVAL_ERROR);
}

#[test]
fn truncation() {
    let repl = Repl::new();
    let input = CString::new("[1,2,3,4,5] ~ [6,7,8,9,10]").unwrap();
    let mut buffer = [0u8; 8];
    let status = unsafe {
        dices_repl_eval(
            repl.0,
            input.as_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    assert_eq!(status, DICES_STATUS_OK | DICES_STATUS_TRUNCATED);
    let text = unsafe { CStr::from_ptr(buffer.as_ptr().cast()) };
    assert!(text.to_bytes().len() < buffer.len());
}

#[test]
fn invalid_arguments() {
    let repl = Repl::new();
    let input = CString::new("1").unwrap();
    let mut buffer = [0u8; 64];

    let status = unsafe {
        dices_repl_eval(
            std::ptr::null_mut(),
            input.as_ptr(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    assert_eq!(status, DICES_STATUS_INVALID_ARG);

    let status = unsafe {
        dices_repl_eval(
            repl.0,
            std::ptr::null(),
            buffer.as_mut_ptr().cast(),
            buffer.len(),
        )
    };
    assert_eq!(status, DICES_STATUS_INVALID_ARG);

    let status =
        unsafe { dices_repl_eval(repl.0, input.as_ptr(), std::ptr::null_mut(), buffer.len()) };
    assert_eq!(status, DICES_STATUS_INVALID_ARG);

    let status = unsafe { dices_repl_eval(repl.0, input.as_ptr(), buffer.as_mut_ptr().cast(), 0) };
    assert_eq!(status, DICES_STATUS_INVALID_ARG);
}

#[test]
fn width_is_honored() {
    let repl = Repl::new();
    unsafe { dices_repl_set_width(repl.0, 10) };
    // With a 10 columns width, a long list must be wrapped
    let (status, text) = repl.eval("[1111, 2222, 3333, 4444]");
    assert_eq!(status, DICES_STATUS_OK);
    assert!(text.contains('\n'), "output should be wrapped: {text:?}");
}

#[test]
fn null_repl_is_ignored() {
    unsafe { dices_repl_reset(std::ptr::null_mut()) };
    unsafe { dices_repl_set_width(std::ptr::null_mut(), 80) };
    unsafe { dices_repl_free(std::ptr::null_mut()) };
}
