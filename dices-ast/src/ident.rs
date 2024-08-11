//! Definitions about `dices` identifiers

use derive_more::derive::{AsRef, Deref, Display};
use lazy_regex::{regex, Lazy, Regex};

pub static IDENT_RE: &Lazy<Regex> = regex!(r"^(?:[a-zA-Z]|_+[a-zA-Z0-9])[_a-zA-Z0-9]*$");

/// A string that is guarantee to be a valid identifier (`r"(?:[a-zA-Z]|_+[a-zA-Z0-9])[_a-zA-Z0-9]*"`)
#[derive(Debug, Display, AsRef, Deref, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct IdentStr(str);

impl IdentStr {
    /// Check if the string is a valid identifier, then convert the reference to
    /// a reference to this type.
    pub fn new(s: &str) -> Option<&Self> {
        if !IDENT_RE.is_match(s) {
            return None;
        }
        Some(unsafe {
            // SAFETY: we checked that IDENT_RE matches
            Self::new_unchecked(s)
        })
    }

    /// Convert to this type with no checks
    ///
    /// ## SAFETY
    /// The user must check that `s` is a match for [`IDENT_RE`]
    pub unsafe fn new_unchecked(s: &str) -> &Self {
        &*(s as *const str as *const Self)
    }

    /// Check if the boxed string is a valid identifier, then convert the reference to
    /// a reference to this type.
    pub fn new_boxed(s: Box<str>) -> Result<Box<Self>, Box<str>> {
        if !IDENT_RE.is_match(&s) {
            return Err(s);
        }
        Ok(unsafe {
            // SAFETY: we checked that IDENT_RE matches
            Self::new_boxed_unchecked(s)
        })
    }

    /// Convert to this type with no checks
    ///
    /// ## SAFETY
    /// The user must check that `s` is a match for [`IDENT_RE`]
    pub unsafe fn new_boxed_unchecked(s: Box<str>) -> Box<Self> {
        Box::from_raw(Box::into_raw(s) as _)
    }
}

impl Clone for Box<IdentStr> {
    fn clone(&self) -> Self {
        let s: &str = self;
        let s: Box<str> = s.into();
        unsafe { IdentStr::new_boxed_unchecked(s) }
    }
}
