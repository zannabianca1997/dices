use anyhow::{Context, Result};
use dices_man::{ManDir, ManItem, ManPage};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote};
use slug::slugify;
use std::{collections::BTreeSet, env, fs, path::PathBuf};

struct ManTests<'m, 'p, T> {
    item: &'m T,
    name_pool: &'p mut BTreeSet<Ident>,
}

impl ManTests<'_, '_, ManDir> {
    fn into_token_stream(self) -> TokenStream {
        // choose a name for the module
        let s = self.item.name;
        let mod_name = rustify(s, self.name_pool);

        let mut names = BTreeSet::new();
        let inners = self.item.content.entries().map(|(_, e)| {
            ManTests {
                item: *e,
                name_pool: &mut names,
            }
            .into_token_stream()
        });

        quote! {
            #[doc = #s]
            #[cfg(test)]
            mod #mod_name {
                #(
                    #inners
                )*
            }
        }
    }
}
impl ManTests<'_, '_, ManItem> {
    fn into_token_stream(self) -> TokenStream {
        match self {
            ManTests {
                item: ManItem::Dir(item),
                name_pool,
            } => ManTests { item, name_pool }.into_token_stream(),
            ManTests {
                item: ManItem::Page(item),
                name_pool,
            } => ManTests { item, name_pool }.into_token_stream(),
        }
    }
}
impl ManTests<'_, '_, ManPage> {
    fn into_token_stream(self) -> TokenStream {
        // choose a name for the module
        let s = self.item.name;
        let mod_name = rustify(s, self.name_pool);

        let mut names = BTreeSet::new();
        let inners = {
            // first, split all the examples
            let mut nodes = vec![self.item.source()];
            let mut examples = vec![];
            while let Some(node) = nodes.pop() {
                match node {
                    markdown::mdast::Node::Code(code) => examples.push(code),
                    _ => nodes.extend(node.children().into_iter().flatten()),
                }
            }

            examples.into_iter()
        }
        .enumerate()
        .filter_map(|(i, e)| {
            // split the tags referring to the tests
            let tags: Vec<_> = e
                .meta
                .as_ref()
                .into_iter()
                .flat_map(|m| m.split_whitespace())
                .filter_map(|t| t.strip_prefix("mantest:"))
                .collect();
            // check that we need to run it as a doc: the language should be `dices` and the tag `mantest:ignore` should be missing
            if !e.lang.as_ref().is_some_and(|l| l == "dices") || tags.contains(&"ignore") {
                return None;
            }

            // find a name
            let name = rustify(
                &e.position
                    .as_ref()
                    .map(|p| format!("line_{}", p.start.line)) // use the line number
                    .unwrap_or_else(|| format!("example_{}", i + 1)), // or fall back to numbering the examples
                &mut names,
            );

            let test = &*e.value;

            // example plot
            Some(quote! {
                #[test]
                fn #name() {
                    crate::_test_impl::test_inner(
                        #test,
                        &[
                            #(
                                #tags
                            ),*
                        ]
                    )
                }
            })
        });

        quote! {
            #[doc = #s]
            #[cfg(test)]
            mod #mod_name {
                #(
                    #inners
                )*
            }
        }
    }
}

fn main() -> Result<()> {
    // Run on all the manual, writing it to a file
    let out_file =
        PathBuf::from(env::var_os("OUT_DIR").expect("The out dir shoul be set in build.rs"))
            .join("man_tests.rs");
    fs::write(
        &out_file,
        ManTests {
            item: &dices_man::MANUAL,
            name_pool: &mut BTreeSet::from([format_ident!("_test_impl")]),
        }
        .into_token_stream()
        .to_string(),
    )
    .context("Cannot write the output rust file")?;
    println!("cargo::rustc-env=MAN_TESTS_RS={}", out_file.display());

    Ok(())
}

fn rustify(s: &str, name_pool: &mut BTreeSet<Ident>) -> Ident {
    // removing all unicode chars, adding a leading _ if starting with a number
    let mut slug = slugify(s).replace("-", "_");
    if slug.starts_with(|ch: char| ch.is_ascii_digit()) {
        slug = "_".to_owned() + &slug;
    }
    let mut ident = format_ident!("{}", slug);
    let mut i: usize = 1;
    while !name_pool.insert(ident.clone()) {
        i += 1;
        ident = format_ident!("{}_{}", slug, i)
    }
    ident
}
