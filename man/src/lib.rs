#![doc = include_str!("../README.md")]

use std::{borrow::Cow, iter::repeat, usize};

use dices_print::{Annotation, markdown::Markdown};
use itertools::Itertools;
use pretty::{DocAllocator, DocBuilder, Pretty};
use rust_embed::Embed;

#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/pages"]
struct Pages;

pub struct ManItem {
    path: Vec<u16>,
    content: Content,
}
enum Content {
    Index,
    Page {
        title: Cow<'static, str>,
        file: Cow<'static, str>,
    },
}

impl ManItem {
    pub const fn root() -> Self {
        Self {
            path: vec![],
            content: Content::Index,
        }
    }

    /// All page descendant from this
    pub fn childs(&self) -> impl Iterator<Item = Self> {
        self.navigate(&[])
    }

    /// Navigate to nested pages
    pub fn navigate(&self, rel_path: &[u16]) -> impl Iterator<Item = Self> {
        Pages::iter()
            .filter_map(|file| {
                let path_str = file.split_whitespace().next().expect(INVALID_TITLE);
                let path: Vec<_> = path_str
                    .split_terminator(['/', '.'])
                    .map(|src| u16::from_str_radix(src, 10))
                    .try_collect()
                    .expect(INVALID_PATH);

                if path.len() <= self.path().len()
                    || path.len() < (self.path().len() + rel_path.len())
                    || !path.starts_with(&self.path())
                    || !path[self.path().len()..].starts_with(rel_path)
                {
                    return None;
                }

                let title_start = file.len() - file[path_str.len()..].trim_start().len();
                let title_end = file.strip_suffix(".md").expect(INVALID_EXT).len();

                let title = match &file {
                    Cow::Borrowed(file) => Cow::Borrowed(&file[title_start..title_end]),
                    Cow::Owned(file) => Cow::Owned(file[title_start..title_end].to_owned()),
                };

                Some(ManItem {
                    path,
                    content: Content::Page { title, file },
                })
            })
            .sorted_by(|a, b| Ord::cmp(a.path(), b.path()))
    }

    /// Title of the item
    pub fn title(&self) -> Cow<'static, str> {
        match &self.content {
            Content::Page { title, .. } => title.clone(),
            Content::Index { .. } => "Index".into(),
        }
    }

    /// Path of this item
    pub fn path(&self) -> &[u16] {
        &self.path
    }

    /// Index of the directory containing this item
    pub fn index(&self) -> Self {
        Self {
            path: self.path.clone(),
            content: Content::Index,
        }
    }

    /// Content of the item
    pub fn content(&self) -> Cow<'static, str> {
        match &self.content {
            Content::Index => self
                .childs()
                .format_with("\n", |i, f| {
                    f(&format_args!(
                        "{}{}. {}",
                        repeat(" ")
                            .take(
                                i.path()
                                    .split_last()
                                    .unwrap()
                                    .1
                                    .iter()
                                    .map(|s| 3 + if *s == 0 { 0 } else { s.ilog10() as usize })
                                    .sum()
                            )
                            .format(""),
                        i.path().last().unwrap(),
                        i.title()
                    ))
                })
                .to_string()
                .into(),
            Content::Page { file, .. } => {
                let data = Pages::get(&file).expect("Page was cancelled").data;

                match data {
                    Cow::Borrowed(s) => str::from_utf8(s).expect(INVALID_FORMAT).into(),
                    Cow::Owned(s) => String::from_utf8(s).expect(INVALID_FORMAT).into(),
                }
            }
        }
    }
}

impl<'a, D> Pretty<'a, D, Annotation> for &ManItem
where
    D: DocAllocator<'a, Annotation> + 'a,
    DocBuilder<'a, D, Annotation>: Clone,
{
    fn pretty(self, allocator: &'a D) -> DocBuilder<'a, D, Annotation> {
        let title = if self.path().is_empty() {
            format!("# {}\n\n", self.title())
        } else {
            format!("# {}. {}\n\n", self.path().iter().format("."), self.title())
        };
        let content = self.content();

        Markdown(title).pretty(allocator).append(Markdown(content))
    }
}

const INVALID_TITLE: &str =
    "All manual pages should have a space separating the position from the title";
const INVALID_PATH: &str =
    "All manual pages should start with a position, and all directory names should be number";
const INVALID_EXT: &str = "All manual pages should have `.md` extension";
const INVALID_FORMAT: &str = "All manual pages should be utf8";
