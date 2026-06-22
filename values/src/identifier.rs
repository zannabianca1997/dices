//! Identifiers

use std::fmt::Display;

use derive_more::{AsRef, Into};
use lazy_regex::regex_is_match;
use phf::phf_set;

use crate::string::ValueString;

static KEYWORDS: phf::Set<&'static str> = phf_set!("d", "kh", "kl", "rh", "rl", "let");

/// A valid identifier
///
/// Due to dices having some operators that are text, identifiers have an
/// additional limitations: they need to not have any operator between two
/// digits or end. `dice` is valid, but not `d`, `d20`, `3d`. `rhs` is parsed as
/// `rh s`, and similarly.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Into, AsRef)]
#[repr(transparent)]
pub struct Identifier(ValueString);

impl Identifier {
    /// Check if a string is a valid identifier
    pub fn is_valid(text: &str) -> bool {
        // Must be a valid C identifier
        regex_is_match!(r"^[_a-zA-Z][_a-zA-Z0-9]*$", text)
        // Must not be made of only underscores
        &&! text.chars().all(|ch| ch == '_')
        // Must not contain word operators at the start or between two
        // digits
        &&! regex_is_match!(r"(?:^|[0-9])(?:d|kh|kl|rh|rl)(?:$|[0-9])", text)
        // Must not be a keyword
        &&! KEYWORDS.contains(text)
    }

    pub fn new(text: ValueString) -> Result<Self, ValueString> {
        if Self::is_valid(&text) {
            Ok(Self(text))
        } else {
            Err(text)
        }
    }

    pub fn new_ref(text: &ValueString) -> Option<&Self> {
        Self::is_valid(&text).then(|| unsafe {
            // Safety: `repr(transparent)`
            &*(text as *const _ as *const _)
        })
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_str().fmt(f)
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        value.0.into()
    }
}
