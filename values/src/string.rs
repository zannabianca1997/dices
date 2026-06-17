//! Strings

use std::{
    borrow::Borrow,
    fmt::{Debug, Display, Write},
    hash::Hash,
    io::Write as _,
    num::ParseIntError,
    ops::Deref,
    slice::SliceIndex,
    sync::Arc,
};

use snafu::{OptionExt, ResultExt, Snafu};
use yoke::Yoke;

/// A string
///
/// Cheaply clonable and sliceable
#[derive(Clone)]
pub struct ValueString(Yoke<&'static str, Option<Arc<String>>>);

impl ValueString {
    /// Create a new string
    pub fn new(value: String) -> Self {
        let content = Yoke::attach_to_cart(Arc::new(value), |cart| &**cart).wrap_cart_in_option();

        Self(content)
    }

    /// Create a new string from static data
    pub const fn new_static(value: &'static str) -> Self {
        let content = Yoke::new_owned(value);

        Self(content)
    }

    /// Value as a string
    pub fn as_str(&self) -> &str {
        self.0.get()
    }

    /// Get a substring
    ///
    /// This will obtain a substring that references the same backing string
    pub fn slice<I>(&self, i: I) -> Option<Self>
    where
        I: SliceIndex<str, Output = str>,
    {
        let inner = self
            .0
            .try_map_project_cloned(|s, _| s.get(i).ok_or(()))
            .ok()?;
        Some(Self(inner))
    }

    /// Concatenate two strings
    pub fn concat(self, other: Self) -> Self {
        if self.is_empty() {
            return other;
        }
        if other.is_empty() {
            return self;
        }

        let mut this = String::from(self);
        this.push_str(other.as_str());

        Self::new(this)
    }
}

#[derive(Debug, Snafu)]
pub enum EscapeError {
    UnexpectedEndOfString,
    UnknownEscapeCode { ch: char },
    InvalidHexEscape { source: ParseIntError },
    UnopenedUnicodeEscape,
    UnclosedUnicodeEscape,
    InvalidCodepoint { code: u32 },
}

impl AsRef<str> for ValueString {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for ValueString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl Borrow<str> for ValueString {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<ValueString> for str {
    fn eq(&self, other: &ValueString) -> bool {
        self.eq(other.as_str())
    }
}
impl PartialEq<&ValueString> for str {
    fn eq(&self, other: &&ValueString) -> bool {
        self.eq(other.as_str())
    }
}
impl<Rhs> PartialEq<Rhs> for ValueString
where
    str: PartialEq<Rhs>,
{
    fn eq(&self, other: &Rhs) -> bool {
        self.as_str().eq(other)
    }
}
impl Eq for ValueString {}

impl PartialOrd<ValueString> for str {
    fn partial_cmp(&self, other: &ValueString) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}
impl PartialOrd<&ValueString> for str {
    fn partial_cmp(&self, other: &&ValueString) -> Option<std::cmp::Ordering> {
        self.partial_cmp(other.as_str())
    }
}
impl<Rhs> PartialOrd<Rhs> for ValueString
where
    str: PartialOrd<Rhs>,
{
    fn partial_cmp(&self, other: &Rhs) -> Option<std::cmp::Ordering> {
        self.as_str().partial_cmp(other)
    }
}
impl Ord for ValueString {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for ValueString {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl Debug for ValueString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ValueString").field(&self.as_str()).finish()
    }
}

impl ValueString {
    /// Remove escape codes from a string, changing them to their escaped value
    pub fn unescape(self) -> Result<Self, EscapeError> {
        let escapes = self.chars().filter(|ch| *ch == '\\').count();
        if escapes == 0 {
            return Ok(self);
        }
        let mut result = String::with_capacity(self.len() - escapes);
        let mut chars = self.chars();

        while let Some(ch) = chars.next() {
            if ch != '\\' {
                result.push(ch);
                continue;
            }

            let Some(escaped) = chars.next() else {
                return Err(EscapeError::UnexpectedEndOfString);
            };

            let unescaped = match escaped {
                '0' => '\0',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\'' => '\'',
                '\\' => '\\',
                'x' => {
                    // Simple `\xHH` single byte hex escape
                    let s = chars.as_str();
                    let [Some(_), Some(_)] = [chars.next(), chars.next()] else {
                        return Err(EscapeError::UnexpectedEndOfString);
                    };
                    let s = &s[..s.len() - chars.as_str().len()];

                    let code = u32::from_str_radix(s, 16).context(InvalidHexEscapeSnafu)?;

                    char::from_u32(code).context(InvalidCodepointSnafu { code })?
                }
                'u' => {
                    // unicode `\u{HHHH}` escape
                    let Some('{') = chars.next() else {
                        return Err(EscapeError::UnopenedUnicodeEscape);
                    };

                    let s = chars.as_str();
                    let Some('}') = chars.find(|ch| *ch == '}') else {
                        return Err(EscapeError::UnopenedUnicodeEscape);
                    };
                    let s = &s[..s.len() - chars.as_str().len() - '}'.len_utf8()];

                    let code = u32::from_str_radix(s, 16).context(InvalidHexEscapeSnafu)?;

                    char::from_u32(code).context(InvalidCodepointSnafu { code })?
                }
                _ => return Err(EscapeError::UnknownEscapeCode { ch }),
            };

            result.push(unescaped);
        }

        Ok(Self::new(result))
    }

    /// Escape the string
    pub fn escape(self, escape: Escape) -> Self {
        let escapes = self.chars().filter(|ch| escape.escapes(*ch)).count();
        if escapes == 0 {
            // Nothing to escape
            return self;
        }

        // Gross estimate: if the string is mostly unescaped this won't
        // overshoot much. If it's all escape it's probably random bytes.
        let mut result = String::with_capacity(self.len() + escapes * 3);

        write!(result, "{}", self.display_content(escape)).unwrap();

        Self::new(result)
    }

    /// Display the string content, escaped
    pub fn display_content(&self, escape: Escape) -> DisplayEscaped<'_> {
        DisplayEscaped {
            content: self.as_str(),
            escape,
        }
    }
}

/// How much to escape a string
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, PartialOrd, Ord)]
pub enum Escape {
    /// Minimal escape to fit a literal
    Minimal,
    /// Escape all control character
    #[default]
    Control,
    /// Escape everything except not-control ascii
    Full,
}

impl Escape {
    /// Check if this escape method escapes the given char
    pub fn escapes(&self, ch: char) -> bool {
        use Escape::*;
        match (self, ch) {
            // Escape and end of string are always escaped
            (_, '\\' | '"') => true,
            // Minimal stops here
            (Minimal, _) => false,
            // Control character are escaped
            (_, ch) if ch.is_control() => true,
            // Control stops here
            (Control, _) => false,
            // Full let only ascii
            (Full, ch) => !ch.is_ascii(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayEscaped<'a> {
    content: &'a str,
    escape: Escape,
}

impl Display for DisplayEscaped<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut hex_scratch = *br"\xHH";
        let mut unicode_scratch = *br"\u{HHHHHHHH}";

        for ch in self.content.chars() {
            if !self.escape.escapes(ch) {
                f.write_char(ch)?;
                continue;
            }

            let escaped = match ch {
                '\0' => r"\0",
                '\n' => r"\n",
                '\r' => r"\r",
                '\t' => r"\t",
                '"' => r#"""#,
                '\'' => r"\'",
                '\\' => r"\",
                '\x00'..='\x7F' => {
                    write!(&mut hex_scratch[2..], "{:02x}", ch as u8).unwrap();
                    str::from_utf8(&hex_scratch[..]).unwrap()
                }
                _ => {
                    let mut scratch = &mut unicode_scratch[3..];
                    write!(scratch, "{:x}}}", ch as u32).unwrap();
                    let unwritten = scratch.len();
                    str::from_utf8(&unicode_scratch[..unicode_scratch.len() - unwritten]).unwrap()
                }
            };

            f.write_str(escaped)?
        }

        Ok(())
    }
}

impl Display for ValueString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.display_content(Escape::default()))
    }
}

impl From<ValueString> for String {
    fn from(value: ValueString) -> Self {
        // If the whole string is covered, copy the string directly
        if value
            .0
            .backing_cart()
            .as_deref()
            .is_some_and(|cart| cart.len() == value.len())
        {
            let cart = Arc::unwrap_or_clone(value.0.into_backing_cart().unwrap());
            return cart;
        }

        // Fallback to copy the substring
        value.as_str().to_owned()
    }
}

impl From<String> for ValueString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl Default for ValueString {
    fn default() -> Self {
        Self::new_static("")
    }
}
