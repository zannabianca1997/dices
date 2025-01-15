//! Definitions about `dices` identifiers
#![allow(unsafe_code)]

use std::ptr;

use derive_more::derive::{AsRef, Deref, Display};
use lazy_regex::{regex, Lazy, Regex};
use phf::phf_set;

static IDENT_RE: &Lazy<Regex> = regex!(r"^(?:[a-zA-Z]|_+[a-zA-Z0-9])[_a-zA-Z0-9]*$");
static KEYWORDS: phf::Set<&'static str> = phf_set!("d", "kh", "kl", "rh", "rl", "let");

pub fn is_valid_ident(s: &str) -> bool {
    IDENT_RE.is_match(s) && !KEYWORDS.contains(s)
}

/// A string that is guarantee to be a valid identifier (`r"(?:[a-zA-Z]|_+[a-zA-Z0-9])[_a-zA-Z0-9]*"`)
#[derive(Debug, Display, AsRef, Deref, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode))]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
#[repr(transparent)]
pub struct IdentStr(str);

impl IdentStr {
    /// Check if the string is a valid identifier, then convert the reference to
    /// a reference to this type.
    #[must_use]
    pub fn new(s: &str) -> Option<&Self> {
        if !is_valid_ident(s) {
            return None;
        }
        Some(unsafe {
            // SAFETY: we checked that IDENT_RE matches
            Self::new_unchecked(s)
        })
    }

    /// Convert to this type with no checks
    ///
    /// # Safety
    /// The user must check that `s` is a match for [`is_valid_ident`]
    #[must_use]
    pub const unsafe fn new_unchecked(s: &str) -> &Self {
        &*(ptr::from_ref(s) as *const Self)
    }

    /// Check if the boxed string is a valid identifier, then convert the reference to
    /// a reference to this type.
    pub fn new_boxed(s: Box<str>) -> Result<Box<Self>, Box<str>> {
        if !is_valid_ident(&s) {
            return Err(s);
        }
        Ok(unsafe {
            // SAFETY: we checked that IDENT_RE matches
            Self::new_boxed_unchecked(s)
        })
    }

    /// Convert to this type with no checks
    ///
    /// # Safety
    /// The user must check that `s` is a match for [`is_valid_ident`]
    #[must_use]
    pub unsafe fn new_boxed_unchecked(s: Box<str>) -> Box<Self> {
        Box::from_raw(Box::into_raw(s) as _)
    }
}

impl ToOwned for IdentStr {
    type Owned = Box<IdentStr>;

    fn to_owned(&self) -> Self::Owned {
        let s: &str = self;
        let s: Box<str> = s.into();
        unsafe { IdentStr::new_boxed_unchecked(s) }
    }
}

impl Clone for Box<IdentStr> {
    fn clone(&self) -> Self {
        let s: &str = self;
        let s: Box<str> = s.into();
        unsafe { IdentStr::new_boxed_unchecked(s) }
    }
}

#[cfg(feature = "bincode")]
impl bincode::Decode for Box<IdentStr> {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        IdentStr::new_boxed(bincode::Decode::decode(decoder)?).map_err(|err| {
            bincode::error::DecodeError::OtherString(format!("Invalid identifier {err}"))
        })
    }
}
#[cfg(feature = "bincode")]
bincode::impl_borrow_decode! {Box<IdentStr>}

#[cfg(feature = "bincode")]
impl<'de> bincode::BorrowDecode<'de> for &'de IdentStr {
    fn borrow_decode<D: bincode::de::BorrowDecoder<'de>>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let ident = bincode::BorrowDecode::borrow_decode(decoder)?;
        IdentStr::new(ident).ok_or_else(|| {
            bincode::error::DecodeError::OtherString(format!("Invalid identifier {ident}"))
        })
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Box<IdentStr> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        IdentStr::new_boxed(serde::Deserialize::deserialize(deserializer)?).map_err(|err| {
            <D::Error as serde::de::Error>::custom(format!("Invalid identifier: {err}"))
        })
    }
}
