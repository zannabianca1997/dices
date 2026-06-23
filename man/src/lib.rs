#![doc = include_str!("../README.md")]


use dices_values::string::ValueString;
use itertools::Itertools;
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/pages"]
struct Pages;

pub struct Item {
    path: Vec<u16>,
    content: Content,
}
enum Content {
    Index,
    Page {
        title: ValueString,
        content: ValueString,
    },
}

impl Item {
    pub const fn root() -> Self {
        Self { path: vec![], content: Content::Index }
    }

    pub fn title(&self) -> ValueString {
        match &self.content {
            Content::Page { title, .. } => title.clone(),
            Content::Index { .. } => "Index".into()
        }
    }

    pub fn path(&self) -> &[u16] {
        &self.path
    }
}
