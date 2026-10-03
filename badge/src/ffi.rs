//! C ABI for `dices-badge`.
//!
//! The consumer-facing declaration of this API is `include/dices_badge.h`;
//! keep the two in sync.

use core::ffi::{c_char, c_int};

use crate::{BadgeRepl, Outcome};

/// Evaluation succeeded.
pub const DICES_STATUS_OK: c_int = 0;
/// The input did not parse.
pub const DICES_STATUS_PARSE_ERROR: c_int = 1;
/// The input parsed but did not evaluate.
pub const DICES_STATUS_EVAL_ERROR: c_int = 2;
/// A pointer argument was null, or the output buffer had no room.
pub const DICES_STATUS_INVALID_ARG: c_int = -1;
/// OR-ed into a status when the output did not fit in the buffer.
pub const DICES_STATUS_TRUNCATED: c_int = 0x100;
/// The manual blob could not be parsed ([`dices_repl_set_manual`]).
pub const DICES_STATUS_MANUAL_ERROR: c_int = 3;

/// Create a new REPL with a random seed and the default wrap width.
///
/// # Safety
///
/// The returned pointer must be released with [`dices_repl_free`].
#[unsafe(no_mangle)]
pub extern "C" fn dices_repl_new() -> *mut BadgeRepl {
    Box::into_raw(Box::new(BadgeRepl::new()))
}

/// Release a REPL. Passing null is ignored.
///
/// # Safety
///
/// `repl` must come from [`dices_repl_new`] and must not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dices_repl_free(repl: *mut BadgeRepl) {
    if !repl.is_null() {
        drop(unsafe { Box::from_raw(repl) });
    }
}

/// Forget all variables of a REPL and re-seed it.
///
/// # Safety
///
/// `repl` must come from [`dices_repl_new`] and still be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dices_repl_reset(repl: *mut BadgeRepl) {
    // SAFETY: the caller guarantees `repl` is valid or null.
    if let Some(repl) = unsafe { repl.as_mut() } {
        repl.reset();
    }
}

/// Set the wrap width used when rendering output.
///
/// # Safety
///
/// `repl` must come from [`dices_repl_new`] and still be valid.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dices_repl_set_width(repl: *mut BadgeRepl, width: c_int) {
    // SAFETY: the caller guarantees `repl` is valid or null.
    if let Some(repl) = (unsafe { repl.as_mut() })
        && width > 0
    {
        repl.set_width(width as usize);
    }
}

/// Load the prerendered manual (a blob produced by `src/bin/prerender.rs`).
///
/// Until this is called, `std.help` answers that pages are unavailable: the
/// badge does not link the markdown renderer that would render them.
///
/// # Safety
///
/// `repl` must come from [`dices_repl_new`] and still be valid; `data` must be
/// readable for `len` bytes.
#[unsafe(no_mangle)]
#[allow(clippy::cast_possible_truncation)]
pub unsafe extern "C" fn dices_repl_set_manual(
    repl: *mut BadgeRepl,
    data: *const u8,
    len: usize,
) -> c_int {
    // SAFETY: the caller guarantees `repl` is valid or null and `data`/`len`
    // describe a readable buffer.
    unsafe {
        let Some(repl) = repl.as_mut() else {
            return DICES_STATUS_INVALID_ARG;
        };
        if data.is_null() {
            return DICES_STATUS_INVALID_ARG;
        }
        let blob = core::slice::from_raw_parts(data, len);
        match repl.set_manual(blob) {
            Ok(()) => DICES_STATUS_OK,
            Err(_) => DICES_STATUS_MANUAL_ERROR,
        }
    }
}

/// Evaluate a `dices` program, possibly spanning multiple lines and statements.
///
/// On any status other than [`DICES_STATUS_INVALID_ARG`], `out` receives a
/// NUL-terminated rendering of the result or of the error. The output is
/// truncated to `out_len - 1` bytes, in which case [`DICES_STATUS_TRUNCATED`]
/// is OR-ed into the returned status.
///
/// # Safety
///
/// `repl` must come from [`dices_repl_new`] and still be valid; `input` must
/// point to a NUL-terminated string; `out` must be writable for `out_len`
/// bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dices_repl_eval(
    repl: *mut BadgeRepl,
    input: *const c_char,
    out: *mut c_char,
    out_len: usize,
) -> c_int {
    if out_len == 0 {
        return DICES_STATUS_INVALID_ARG;
    }
    // SAFETY: the caller guarantees a valid `repl`, a NUL-terminated `input`,
    // and a writable `out` of `out_len` bytes (checked to be non-zero above).
    unsafe {
        let Some(repl) = repl.as_mut() else {
            return DICES_STATUS_INVALID_ARG;
        };
        if input.is_null() || out.is_null() {
            return DICES_STATUS_INVALID_ARG;
        }

        let input = core::ffi::CStr::from_ptr(input);
        let output = repl.eval(&input.to_string_lossy());

        let bytes = output.text.as_bytes();
        let writable = out_len - 1;
        let copy_len = bytes.len().min(writable);
        core::ptr::copy_nonoverlapping(bytes.as_ptr().cast::<c_char>(), out, copy_len);
        *out.add(copy_len) = 0;

        let status = match output.outcome {
            Outcome::Success => DICES_STATUS_OK,
            Outcome::ParseError => DICES_STATUS_PARSE_ERROR,
            Outcome::EvalError => DICES_STATUS_EVAL_ERROR,
        };
        status
            | if bytes.len() > writable {
                DICES_STATUS_TRUNCATED
            } else {
                0
            }
    }
}
