//! This package contains  the manual pages for `dices`

#![feature(never_type)]

use std::sync::OnceLock;

use markdown::{mdast::Node, to_mdast, ParseOptions};

/// A page of the manual
pub struct ManPage {
    /// The name of the page
    pub name: &'static str,
    /// The content of the page
    pub content: &'static str,
    /// The markdown ast of the page, if parsed
    ast: OnceLock<Node>,
}
impl ManPage {
    const fn new(name: &'static str, content: &'static str) -> Self {
        Self {
            name,
            content,
            ast: OnceLock::new(),
        }
    }

    pub fn ast(&self) -> &Node {
        self.ast
            .get_or_init(|| to_mdast(&self.content, &ParseOptions::default()).unwrap())
    }
}

/// A subdirectory of the manual
pub struct ManDir {
    /// The name of the subdirectory
    pub name: &'static str,
    /// The content of the subdirectory
    pub content: phf::OrderedMap<&'static str, &'static ManItem>,
}

/// A item of the manual
pub enum ManItem {
    /// A single page
    Page(ManPage),
    /// A directory of items
    Dir(ManDir),
}

pub static MANUAL: ManDir = include!(env!("MANUAL_RS"));

pub mod example;
