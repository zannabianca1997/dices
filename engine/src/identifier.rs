//! Identifiers

use std::{borrow::Borrow, fmt::Display, ops::Deref, rc::Rc, sync::Arc};

use lazy_regex::{lazy_regex, Lazy, Regex};
use thiserror::Error;

static KEYWORDS: phf::Set<&'static str> = phf::phf_set! {
    "let", "true", "false", "null", "d"
};

static IDENT_RE: Lazy<Regex> = lazy_regex!(r"^(?:_+[^_\W]|[^_\d\W])\w*$");

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct IdentStr(str);

impl IdentStr {
    /// Check if a `str` contains a valid identifier,
    /// and convert the reference if valid.
    ///
    /// ```
    /// # use engine::identifier::IdentStr;
    /// assert!(IdentStr::new("ident_0").is_some_and(|ident| ident == "ident_0"));
    /// assert!(IdentStr::new("let").is_none());
    /// ```
    pub fn new(value: &str) -> Option<&Self> {
        // check if it is a valid identifier, and not a valid keyword
        if !KEYWORDS.contains(value) && IDENT_RE.is_match(value) {
            Some(Self::new_unchecked(value))
        } else {
            None
        }
    }

    /// Check if a `str` contains a valid identifier,
    /// and convert the reference if valid.
    /// Return the reason if invalid
    ///
    /// ```
    /// # use engine::identifier::{IdentStr,NotIdentErr};
    /// assert!(IdentStr::try_new("ident_0").is_ok_and(|ident| ident == "ident_0"));
    /// assert!(IdentStr::try_new("let").is_err_and(|err| err == NotIdentErr::Keyword));
    /// ```
    pub fn try_new(value: &str) -> Result<&Self, NotIdentErr> {
        // check if it is a valid identifier, and not a valid keyword
        if !KEYWORDS.contains(value) {
            if IDENT_RE.is_match(value) {
                Ok(Self::new_unchecked(value))
            } else {
                Err(NotIdentErr::InvalidIdent)
            }
        } else {
            Err(NotIdentErr::Keyword)
        }
    }

    /// Check if a `Box<str>` contains a valid identifier,
    /// and convert the box if valid, without allocating.
    ///
    /// ```
    /// # use engine::identifier::IdentStr;
    /// assert!(IdentStr::new_boxed("ident_0".into()).is_ok_and(|ident| *ident == *"ident_0"));
    /// assert!(IdentStr::new_boxed("let".into()).is_err_and(|boxed| *boxed == *"let"));
    /// ```
    pub fn new_boxed(value: Box<str>) -> Result<Box<Self>, Box<str>> {
        // check if it is a valid identifier, and not a valid keyword
        if !KEYWORDS.contains(&*value) && IDENT_RE.is_match(&*value) {
            Ok(Self::new_boxed_unchecked(value))
        } else {
            Err(value)
        }
    }

    /// Check if a `Box<str>` contains a valid identifier,
    /// and convert the box if valid, without allocating.
    /// Return the reason if invalid
    ///
    /// ```
    /// # use engine::identifier::{IdentStr, NotIdentErr};
    /// assert!(IdentStr::try_new_boxed("ident_0".into()).is_ok_and(|ident| *ident == *"ident_0"));
    /// assert!(IdentStr::try_new_boxed("let".into()).is_err_and(|(err, boxed)| *boxed == *"let" && err == NotIdentErr::Keyword));
    /// ```
    pub fn try_new_boxed(value: Box<str>) -> Result<Box<Self>, (NotIdentErr, Box<str>)> {
        // check if it is a valid identifier, and not a valid keyword
        if !KEYWORDS.contains(&*value) {
            if IDENT_RE.is_match(&*value) {
                Ok(Self::new_boxed_unchecked(value))
            } else {
                Err((NotIdentErr::InvalidIdent, value))
            }
        } else {
            Err((NotIdentErr::Keyword, value))
        }
    }

    /// Check if a `Rc<str>` contains a valid identifier,
    /// and convert the rc if valid, without allocating.
    /// This is sound, as `Rc` inner is immutable, so
    /// no one can change the identifier to a invalid one
    ///
    /// ```
    /// # use engine::identifier::IdentStr;
    /// assert!(IdentStr::new_rc("ident_0".into()).is_ok_and(|ident| *ident == *"ident_0"));
    /// assert!(IdentStr::new_rc("let".into()).is_err_and(|rc| *rc == *"let"));
    /// ```
    pub fn new_rc(value: Rc<str>) -> Result<Rc<Self>, Rc<str>> {
        // check if it is a valid identifier, and not a valid keyword
        if !KEYWORDS.contains(&*value) && IDENT_RE.is_match(&*value) {
            Ok(Self::new_rc_unchecked(value))
        } else {
            Err(value)
        }
    }

    /// Check if a `Rc<str>` contains a valid identifier,
    /// and convert the rc if valid, without allocating.
    /// This is sound, as `Rc` inner is immutable, so
    /// no one can change the identifier to a invalid one
    /// Return the reason if invalid
    ///
    /// ```
    /// # use engine::identifier::{IdentStr, NotIdentErr};
    /// assert!(IdentStr::try_new_rc("ident_0".into()).is_ok_and(|ident| *ident == *"ident_0"));
    /// assert!(IdentStr::try_new_rc("let".into()).is_err_and(|(err, rc)| *rc == *"let" && err == NotIdentErr::Keyword));
    /// ```
    pub fn try_new_rc(value: Rc<str>) -> Result<Rc<Self>, (NotIdentErr, Rc<str>)> {
        // check if it is a valid identifier, and not a valid keyword
        if !KEYWORDS.contains(&*value) {
            if IDENT_RE.is_match(&*value) {
                Ok(Self::new_rc_unchecked(value))
            } else {
                Err((NotIdentErr::InvalidIdent, value))
            }
        } else {
            Err((NotIdentErr::Keyword, value))
        }
    }

    /// Create an identifier without checking the content
    fn new_unchecked(value: &str) -> &Self {
        unsafe {
            // SAFETY: `#[repr(transparent)]` guarantee the two types have the same memory layout
            &*(value as *const str as *const IdentStr)
        }
    }
    /// Create a boxed identifier without checking the content
    fn new_boxed_unchecked(value: Box<str>) -> Box<Self> {
        unsafe {
            // SAFETY: `#[repr(transparent)]` guarantee the two types have the same memory layout
            Box::from_raw(Box::into_raw(value) as *mut IdentStr)
        }
    }
    /// Create a rc identifier without checking the content
    fn new_rc_unchecked(value: Rc<str>) -> Rc<Self> {
        unsafe {
            // SAFETY: `#[repr(transparent)]` guarantee the two types have the same memory layout
            Rc::from_raw(Rc::into_raw(value) as *mut IdentStr)
        }
    }
}

#[derive(Clone, Copy, Debug, Error, PartialEq, Eq, PartialOrd, Ord)]
pub enum NotIdentErr {
    #[error("Not a valid identifier")]
    InvalidIdent,
    #[error("Is a valid keyword")]
    Keyword,
}

// conversion to str

impl Deref for IdentStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Borrow<str> for IdentStr {
    fn borrow(&self) -> &str {
        &self.0
    }
}
impl AsRef<str> for IdentStr {
    #[inline(always)]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'a> From<&'a IdentStr> for &'a str {
    fn from(value: &'a IdentStr) -> Self {
        value
    }
}
impl From<Box<IdentStr>> for Box<str> {
    fn from(value: Box<IdentStr>) -> Self {
        unsafe {
            // SAFETY: `#[repr(transparent)]` guarantee the two types have the same memory layout
            Box::from_raw(Box::into_raw(value) as *mut str)
        }
    }
}

// fallible conversion from str

impl<'a> TryFrom<&'a str> for &'a IdentStr {
    type Error = NotIdentErr;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        IdentStr::try_new(value)
    }
}

impl TryFrom<Box<str>> for Box<IdentStr> {
    type Error = (NotIdentErr, Box<str>);

    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        IdentStr::try_new_boxed(value)
    }
}

// boxing

impl<'a> From<&'a IdentStr> for Box<IdentStr> {
    fn from(value: &'a IdentStr) -> Self {
        // we already know this `str` is a valid identifier
        IdentStr::new_boxed_unchecked(
            // Box<str> already has a way to move a str into the heap, let's use it
            Box::<str>::from(&**value),
        )
    }
}
impl<'a> From<&'a IdentStr> for Rc<IdentStr> {
    fn from(value: &'a IdentStr) -> Self {
        // we already know this `str` is a valid identifier
        IdentStr::new_rc_unchecked(
            // Rc<str> already has a way to move a str into the heap, let's use it
            Rc::<str>::from(&**value),
        )
    }
}

// display implementation

impl Display for IdentStr {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <str as Display>::fmt(&self.0, f)
    }
}

// comparison with other string-like types

macro_rules! comparisons {
    ($T:ty) => {
        impl PartialEq<$T> for IdentStr {
            #[inline(always)]
            fn eq(&self, other: &$T) -> bool {
                str::eq(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn ne(&self, other: &$T) -> bool {
                str::ne(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
        }
        impl PartialEq<IdentStr> for $T {
            #[inline(always)]
            fn eq(&self, other: &IdentStr) -> bool {
                str::eq(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn ne(&self, other: &IdentStr) -> bool {
                str::ne(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
        }
        impl PartialOrd<$T> for IdentStr {
            #[inline(always)]
            fn partial_cmp(&self, other: &$T) -> Option<std::cmp::Ordering> {
                str::partial_cmp(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn lt(&self, other: &$T) -> bool {
                str::lt(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn le(&self, other: &$T) -> bool {
                str::le(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn gt(&self, other: &$T) -> bool {
                str::gt(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn ge(&self, other: &$T) -> bool {
                str::ge(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
        }
        impl PartialOrd<IdentStr> for $T {
            #[inline(always)]
            fn partial_cmp(&self, other: &IdentStr) -> Option<std::cmp::Ordering> {
                str::partial_cmp(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn lt(&self, other: &IdentStr) -> bool {
                str::lt(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn le(&self, other: &IdentStr) -> bool {
                str::le(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn gt(&self, other: &IdentStr) -> bool {
                str::gt(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
            #[inline(always)]
            fn ge(&self, other: &IdentStr) -> bool {
                str::ge(AsRef::<str>::as_ref(&self), AsRef::<str>::as_ref(&other))
            }
        }
    };
}

comparisons!(String);
comparisons!(str);
comparisons!(Arc<str>);
comparisons!(Rc<str>);
comparisons!(Box<str>);

// cloning boxed identifier

impl Clone for Box<IdentStr> {
    fn clone(&self) -> Self {
        (&**self).into()
    }
}

// Owning

impl ToOwned for IdentStr {
    type Owned = Box<IdentStr>;

    fn to_owned(&self) -> Self::Owned {
        self.into()
    }
}

#[cfg(feature = "serde")]
mod serde {
    use serde::{de::Error as _, Deserialize, Serialize};

    use super::IdentStr;

    impl Serialize for IdentStr {
        #[inline(always)]
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ::serde::Serializer,
        {
            <str as Serialize>::serialize(&self.0, serializer)
        }
    }

    impl<'de> Deserialize<'de> for &'de IdentStr {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            IdentStr::try_new(Deserialize::deserialize(deserializer)?).map_err(D::Error::custom)
        }
    }
    impl<'de> Deserialize<'de> for Box<IdentStr> {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            IdentStr::try_new_boxed(Deserialize::deserialize(deserializer)?)
                .map_err(|(err, _)| D::Error::custom(err))
        }
    }
}
