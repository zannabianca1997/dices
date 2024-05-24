use std::{borrow::Borrow, fmt::Display, rc::Rc};

use lazy_regex::regex_is_match;

/// `dices` identifier
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(into = "Rc<str>", try_from = "Rc<str>")
)]
pub struct DIdentifier(Rc<str>);

impl DIdentifier {
    pub fn new<V>(value: V) -> Option<Self>
    where
        V: Into<Rc<str>> + Borrow<str>,
    {
        if Self::is_valid(value.borrow()) {
            Some(Self(value.into()))
        } else {
            None
        }
    }

    fn is_valid(value: &str) -> bool {
        regex_is_match!(r"^(?:_+[a-zA-Z0-9]|[a-zA-Z])[_a-zA-Z0-9]*$", value)
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DIdentifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl From<DIdentifier> for Rc<str> {
    fn from(value: DIdentifier) -> Self {
        value.0
    }
}
impl From<DIdentifier> for String {
    fn from(value: DIdentifier) -> Self {
        value.0.to_string()
    }
}

impl TryFrom<String> for DIdentifier {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(Self(value.into()))
        } else {
            Err(value)
        }
    }
}
impl<'a> TryFrom<&'a str> for DIdentifier {
    type Error = &'a str;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        if Self::is_valid(value) {
            Ok(Self(value.into()))
        } else {
            Err(value)
        }
    }
}
impl TryFrom<Rc<str>> for DIdentifier {
    type Error = Rc<str>;

    fn try_from(value: Rc<str>) -> Result<Self, Self::Error> {
        if Self::is_valid(&value) {
            Ok(Self(value))
        } else {
            Err(value)
        }
    }
}
