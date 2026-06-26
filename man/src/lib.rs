#![doc = include_str!("../README.md")]

use std::{borrow::Cow, ops::Deref};

pub use registry::Manual;
pub type Descendants<'p> = registry::Descendant<'p, Cow<'static, [PathComponent]>>;

pub mod registry;

pub type PathComponent = u16;

#[derive(Debug, Clone)]
pub struct ManPage {
    manual: Manual,
    path: Cow<'static, [PathComponent]>,
    content: ManPageContent,
}

#[derive(Debug, Clone)]
pub struct ManPageContent {
    title: Cow<'static, str>,
    content: Cow<'static, str>,
}

impl ManPage {
    /// Path of this page in the manual
    pub fn path(&self) -> &[PathComponent] {
        &self.path
    }

    /// Iter all pages in the manual that descends from this, inclusive of itself
    ///
    /// The iteration order is unspecified
    pub fn descendant(&self) -> Descendants<'_> {
        self.manual.descendants(&self.path)
    }

    /// Get the handler to the manual
    pub fn manual(&self) -> Manual {
        self.manual
    }
}

impl ManPageContent {
    pub fn new(title: impl Into<Cow<'static, str>>, content: impl Into<Cow<'static, str>>) -> Self {
        Self {
            title: title.into(),
            content: content.into(),
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn content(&self) -> &str {
        &self.content
    }
}

impl Deref for ManPage {
    type Target = ManPageContent;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl PartialEq for ManPage {
    fn eq(&self, other: &Self) -> bool {
        self.path() == other.path()
    }
}
impl Eq for ManPage {}
impl PartialOrd for ManPage {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for ManPage {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path().cmp(other.path())
    }
}
