#![feature(iterator_try_collect)]
#![feature(box_patterns)]
#![feature(vec_pop_if)]

use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use anyhow::{bail, Context};
use lazy_regex::regex_captures;
use quote::quote;
use serde::{de::Error as _, Deserialize};
use serde_yml::{value::TaggedValue, Value};

#[derive(Debug, Clone)]
struct Manual {
    items: Vec<ManualItem>,
}

impl Manual {
    fn read(dir: impl AsRef<Path>) -> anyhow::Result<Self> {
        let ManualIndex { items } = serde_yml::from_reader(
            File::open(dir.as_ref().join("index.yaml")).context("Cannot open main index file")?,
        )
        .context("Cannot parse main index file")?;

        Ok(Self {
            items: items
                .into_iter()
                .map(|item| ManualItem::read(dir.as_ref(), item))
                .try_collect()?,
        })
    }

    fn emit(&self, index: &mut phf_codegen::Map<String>) {
        for item in &self.items {
            if let ManualItem::Index = item {
                let title = "`dices` manual";
                let content = make_index(title, &self.items);

                index.entry(
                    "index".to_string(),
                    &quote! {
                        Page {
                            title: #title,
                            content: Cow::Borrowed(#content)
                        }
                    }
                    .to_string(),
                );
            } else {
                item.emit("", index)
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct ManualIndex {
    items: Vec<IndexItem>,
}

#[derive(Debug, Clone)]
enum IndexItem {
    Index,
    Path(PathBuf),
}
impl<'de> Deserialize<'de> for IndexItem {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        match Value::deserialize(deserializer)? {
            Value::String(path) => Ok(Self::Path(path.into())),
            Value::Tagged(box TaggedValue {
                tag,
                value: Value::String(path),
            }) if tag.string == "Path" => Ok(Self::Path(path.into())),
            Value::Tagged(box TaggedValue {
                tag,
                value: Value::Null,
            }) if tag.string == "Index" => Ok(Self::Index),
            _ => Err(D::Error::custom("Expected path or !Index")),
        }
    }
}

#[derive(Debug, Clone)]
enum ManualItem {
    Index,
    Page(Page),
    Nested(Nested),
}
impl ManualItem {
    fn read(base: &Path, item: IndexItem) -> anyhow::Result<ManualItem> {
        Ok(match item {
            IndexItem::Index => Self::Index,
            IndexItem::Path(path) => {
                let path = base.join(path);
                if path.is_file() {
                    Self::Page(Page::read(path)?)
                } else if path.is_dir() {
                    Self::Nested(Nested::read(path)?)
                } else {
                    bail!("{} is not a valid item", path.display())
                }
            }
        })
    }

    fn emit(&self, path: &str, index: &mut phf_codegen::Map<String>) {
        match self {
            ManualItem::Index => unreachable!(),
            ManualItem::Page(page) => page.emit(path, index),
            ManualItem::Nested(nested) => nested.emit(path, index),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
struct NestedIndex {
    #[serde(flatten)]
    front_matter: NestedFrontMatter,
    items: Vec<IndexItem>,
}

#[derive(Debug, Clone)]
struct Nested {
    name: String,
    front_matter: NestedFrontMatter,
    items: Vec<ManualItem>,
}
impl Nested {
    fn read(path: PathBuf) -> anyhow::Result<Nested> {
        let NestedIndex {
            mut front_matter,
            items,
        } = serde_yml::from_reader(
            File::open(path.join("index.yaml")).context("Cannot open nested index")?,
        )
        .context("Cannot parse nested index")?;
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::format_err!("Invalid name for nested"))?
            .to_owned();
        front_matter.title = Some(front_matter.title.unwrap_or_else(|| name.clone()));
        Ok(Self {
            name,
            front_matter,
            items: items
                .into_iter()
                .map(|item| ManualItem::read(&path, item))
                .try_collect()?,
        })
    }

    fn emit(&self, path: &str, index: &mut phf_codegen::Map<String>) {
        let path = if path.is_empty() {
            self.name.clone()
        } else {
            format!("{path}/{}", self.name)
        };

        for item in &self.items {
            item.emit(&path, index)
        }

        let title = &**self.front_matter.title.as_ref().unwrap();
        let content = make_index(title, &self.items);
        index.entry(
            path,
            &quote! {
                Page {
                    title: #title,
                    content: Cow::Borrowed(#content)
                }
            }
            .to_string(),
        );
    }
}

fn make_index(title: &str, items: &[ManualItem]) -> String {
    let mut buf = String::new();

    buf.push_str("# Index of ");
    buf.push_str(title);
    buf.push_str("\n\n");

    let mut stack = vec![(0, items)];

    while let Some((indent, page)) = {
        while stack.pop_if(|(_, i)| i.is_empty()).is_some() {}
        stack.last_mut().map(|(indent, items)| {
            (*indent, {
                let (item, new_items) = items.split_first().unwrap();
                *items = new_items;
                item
            })
        })
    } {
        for _ in 0..indent {
            buf.push_str(" ")
        }
        buf.push_str("* ");
        match page {
            ManualItem::Index => buf.push_str("`index`"),
            ManualItem::Page(Page {
                name,
                front_matter: PageFrontMatter { title, .. },
                ..
            })
            | ManualItem::Nested(Nested {
                name,
                front_matter: NestedFrontMatter { title, .. },
                ..
            }) => {
                if let Some(title) = title.as_deref() {
                    if title != name {
                        buf.push('`');
                        buf.push_str(name);
                        buf.push_str("`: ");
                        buf.push_str(title);
                    } else {
                        buf.push('`');
                        buf.push_str(name);
                        buf.push('`');
                    }
                } else {
                    buf.push('`');
                    buf.push_str(name);
                    buf.push('`');
                }
            }
        }
        buf.push('\n');

        if let ManualItem::Nested(Nested { items, .. }) = page {
            stack.push((indent + 1, items))
        }
    }

    buf
}
/*
fn find_free_hash(path: impl Hash, hashes: &mut BTreeSet<[u8; 5]>) -> String {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    let [_, _, _, mut hash @ ..] = hasher.finish().to_le_bytes();
    let mut counter = 0;
    let hash = loop {
        if hashes.insert(hash) {
            break hash;
        }
        let mut hasher = hasher.clone();
        counter.hash(&mut hasher);
        counter += 1;
        let [_, _, _, new_hash @ ..] = hasher.finish().to_le_bytes();
        hash = new_hash
    };
    data_encoding::BASE32_NOPAD.encode(&hash)
}
 */
#[derive(Debug, Clone)]
struct Page {
    name: String,
    front_matter: PageFrontMatter,
    content: String,
}
impl Page {
    fn read(path: PathBuf) -> anyhow::Result<Page> {
        let content = fs::read_to_string(&path).context("Cannot read the page")?;
        let (mut front_matter, content) = match regex_captures!(
            r"^[^\S\n]*---[^\S\n]*\n((?:.*\n)*?)[^\S\n]*---[^\S\n]*(?:\n|$)((?:.*\n)*.*)$",
            &content
        ) {
            Some((_, front, content)) => (
                serde_yml::from_str(&front).context("Cannot parse front matter")?,
                content,
            ),
            None => (PageFrontMatter::default(), &*content),
        };
        let name = path
            .file_stem()
            .and_then(|n| n.to_str())
            .ok_or_else(|| anyhow::format_err!("Invalid name for page"))?
            .to_owned();
        front_matter.title = Some(front_matter.title.unwrap_or_else(|| name.clone()));
        Ok(Page {
            name,
            front_matter,
            content: content.to_string(),
        })
    }

    fn emit(&self, path: &str, index: &mut phf_codegen::Map<String>) {
        let path = if path.is_empty() {
            self.name.clone()
        } else {
            format!("{path}/{}", self.name)
        };

        let title = &**self.front_matter.title.as_ref().unwrap();
        let content = &*self.content;

        index.entry(
            path,
            &quote! {
                Page {
                    title: #title,
                    content: Cow::Borrowed(#content)
                }
            }
            .to_string(),
        );
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
struct NestedFrontMatter {
    title: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct PageFrontMatter {
    title: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let index_path = Path::new(&env::var_os("OUT_DIR").unwrap()).join("index.rs");
    let mut index_file =
        BufWriter::new(File::create(&index_path).context("Cannot create index file")?);

    let mut index = phf_codegen::Map::<String>::new();

    // parsing the manual
    let manual = Manual::read(Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("pages"))
        .context("Cannot parse the manual")?;

    manual.emit(&mut index);

    writeln!(&mut index_file, "{}", index.build()).context("Cannot write index file")?;
    Ok(())
}
