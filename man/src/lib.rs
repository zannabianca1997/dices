#![doc = include_str!("../README.md")]

use std::{borrow::Cow, iter::Filter, ops::Deref};

pub use registry::Manual;
use slugify::slugify;
use url::Url;

use crate::registry::linked;
pub type Descendants<'p> = registry::Descendant<'p, Cow<'static, [PathComponent]>>;

pub mod examples;
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
    pub fn descendants(&self) -> Descendants<'_> {
        self.manual.descendants(&self.path)
    }

    /// Iter all pages in the manual that directly descends from this
    ///
    /// The iteration order is unspecified
    pub fn children(&self) -> Filter<Descendants<'_>, impl Fn(&ManPage) -> bool> {
        let is_child = |p: &ManPage| p.path().len() == self.path().len() + 1;
        self.manual.descendants(&self.path).filter(is_child)
    }

    /// Get the handler to the manual
    pub fn manual(&self) -> Manual {
        self.manual
    }

    pub fn url(&self) -> Url {
        let mut url = Url::parse("https://dices.zannabianca1997.site/man").unwrap();

        url.path_segments_mut()
            .unwrap()
            .extend(self.path().iter().map(|s| s.to_string()))
            .push(&format!("{}.html", slugify!(self.title())));

        url
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
    pub fn static_title(&self) -> Result<&'static str, &str> {
        match &self.title {
            Cow::Borrowed(s) => Ok(s),
            Cow::Owned(s) => Err(s),
        }
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

/// Root manual page
#[linked::distributed_slice(linked::LINKED_PAGES)]
static ROOT_PAGE: linked::LinkedPage = linked::LinkedPage {
    path: &[],
    title: "Manual for `dices`",
    content: "Use `help([1])`, `help([2,1])`, etc. to see specific pages.",
};
