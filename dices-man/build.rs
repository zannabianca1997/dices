#![feature(iterator_try_collect)]

use core::str;
use std::{
    env::{self, var_os},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use lazy_regex::regex_captures;
use phf_codegen::OrderedMap;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use serde::Deserialize;

/// A page of the manual
struct ManPage {
    /// The name of the page
    name: String,
    /// The content of the page
    content: String,
}
impl ToTokens for ManPage {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &*self.name;
        let content = &*self.content;

        quote!(ManPage::new(#name, #content)).to_tokens(tokens)
    }
}

/// A subdirectory of the manual
struct ManDir {
    /// The name of the subdirectory
    name: String,
    /// The content of the subdirectory
    content: Vec<(String, ManItem)>,
}
impl ToTokens for ManDir {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = &*self.name;
        let mut content = OrderedMap::new();
        for (name, item) in &self.content {
            content.entry(
                name,
                &quote!(
                    {
                        static ITEM: ManItem = #item;
                        &ITEM
                    }
                )
                .to_string(),
            );
        }
        let content: TokenStream = content
            .build()
            .to_string()
            .parse()
            .expect("The builder should produce valid rust");
        quote! (ManDir::new(#name, #content)).to_tokens(tokens)
    }
}

/// A item of the manual
enum ManItem {
    /// A single page
    Page(ManPage),
    /// Index of the directory
    Index,
    /// A directory of items
    Dir(ManDir),
}
impl ManItem {
    fn name(&self) -> &str {
        let (ManItem::Page(ManPage { name, .. }) | ManItem::Dir(ManDir { name, .. })) = self else {
            return "Index";
        };
        &name
    }
}
impl ToTokens for ManItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            ManItem::Page(page) => quote! (ManItem::Page(#page)),
            ManItem::Dir(dir) => quote!(ManItem::Dir(#dir)),
            ManItem::Index => quote!(ManItem::Index(ManIndex::new())),
        }
        .to_tokens(tokens)
    }
}

fn main() -> Result<()> {
    // Find the manual path
    let man_path = PathBuf::from(var_os("DICES_MAN").unwrap_or("man".into()));
    println!("cargo::rerun-if-env-changed=DICES_MAN");
    println!("Reading the manual from {}", man_path.display());
    // Read the manual
    let man_root = read_item(&man_path).context("Cannot read the manual")?;
    let ManItem::Dir(man_root) = man_root else {
        bail!("The manual root must be a directory")
    };
    // Write it to a file
    let out_file =
        PathBuf::from(env::var_os("OUT_DIR").expect("The out dir shoul be set in build.rs"))
            .join("man.rs");
    fs::write(&out_file, man_root.into_token_stream().to_string())
        .context("Cannot write the output rust file")?;
    println!("cargo::rustc-env=MANUAL_RS={}", out_file.display());

    Ok(())
}

fn read_item(path: &Path) -> Result<ManItem> {
    // first: is it a directory or a file?
    if let Ok(content) = fs::read_to_string(path) {
        // parsing the page content
        Ok(ManItem::Page(read_page(path, content).context(format!(
            "Cannot read manual page {}",
            path.display()
        ))?))
    } else if let Ok(index) = fs::read_to_string(path.join("index.yml")) {
        // parsing the directory
        Ok(ManItem::Dir(read_dir(path, index).context(format!(
            "Cannot read manual dir {}",
            path.display()
        ))?))
    } else {
        bail!(
            "Cannot parse {}: is not a file, and neither is {}",
            path.display(),
            path.join("index.yml").display()
        )
    }
}

#[derive(Deserialize, Default)]
struct FrontMatter {
    name: Option<String>,
}

fn read_page(path: &Path, content: String) -> Result<ManPage> {
    println!("cargo::rerun-if-changed={}", path.display());

    // read the file content
    let (FrontMatter { name }, content) = regex_captures!(
        r"\A\s*---((?:.|\n)*)---\s*$(?:\r\n|\n)?((?:.|\n)*)\z"m,
        &content
    )
    .map(|(_, front, content)| {
        serde_yaml::from_str(front)
            .context("Cannot parse yaml frontmatter")
            .map(|front| (front, content))
    })
    .unwrap_or_else(|| Ok((FrontMatter::default(), &*content)))?;
    let name = name.unwrap_or_else(|| {
        path.file_stem()
            .expect("Every file name should have a stem")
            .to_string_lossy()
            .into_owned()
    });
    let content = content.to_owned();
    Ok(ManPage { name, content })
}

#[derive(Deserialize)]
struct Index {
    name: Option<String>,
    index: Vec<String>,
}

fn read_dir(path: &Path, index: String) -> Result<ManDir> {
    println!("cargo::rerun-if-changed={}", path.display());
    println!(
        "cargo::rerun-if-changed={}",
        path.join("index.yml").display()
    );

    let Index { name, index } = serde_yaml::from_str(&index).context("Cannot parse index file")?;
    let name = name.unwrap_or_else(|| {
        path.file_stem()
            .expect("Every file name should have a stem")
            .to_string_lossy()
            .into_owned()
    });
    let content = index
        .into_iter()
        .map(|item_path| -> Result<_> {
            if item_path == "index.yml" {
                Ok(("index".to_owned(), ManItem::Index))
            } else {
                let path = path.join(item_path);
                let item = read_item(&path)?;
                Ok((item.name().to_owned(), item))
            }
        })
        .try_collect()?;
    Ok(ManDir { name, content })
}
