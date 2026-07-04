use std::{env, fs, path::PathBuf};

use dices_man::{
    ManPage, Manual,
    examples::{Command, Example},
};

// Ensure that `std` is brought in, so the man pages for it are present
use dices_std as _;
use itertools::Itertools;
use proc_macro2::TokenStream;
use pulldown_cmark::{CodeBlockKind, Event, Parser, Tag, TagEnd};
use quote::{format_ident, quote};

fn main() {
    let root = Manual::new().root();

    let tests = build_tests_for(&root);

    let out_file_content = syn::parse2(tests.clone())
        .map(|file| prettyplease::unparse(&file))
        .unwrap_or_else(|err| {
            cargo_build::warning!("Error in formatting output: {err}");
            tests.to_string()
        });

    let mut out = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    out.push("tests.rs");

    fs::write(&out, out_file_content).expect("Cannot write out file");
    cargo_build::rustc_env!("COLLECTED_TESTS" = (out.display()))
}

fn build_tests_for(page: &ManPage) -> TokenStream {
    let mut count = 1;
    let tests = Parser::new(page.content())
        .array_windows()
        .filter_map(|window| {
            // Match code blocks
            let [
                Event::Start(Tag::CodeBlock(kind)),
                Event::Text(text),
                Event::End(TagEnd::CodeBlock),
            ] = window
            else {
                return None;
            };

            let (language, tags) = match &kind {
                CodeBlockKind::Fenced(cow_str) => {
                    match cow_str.trim_start().split_once(char::is_whitespace) {
                        Some((a, b)) => (Some(a.trim_end()), Some(b.trim())),
                        None => (Some(cow_str.trim()).filter(|s| !s.is_empty()), None),
                    }
                }
                CodeBlockKind::Indented => (None, None),
            };

            if !matches!(language, None | Some("dices") | Some("dices-example")) {
                return None;
            }

            let Example { tags, commands } = Example::new(tags.unwrap_or_default(), &text);

            let commands = commands.into_iter().map(
                |Command {
                     hidden,
                     command,
                     response,
                 }| {
                    quote! {
                        ::dices_man::examples::Command {
                            hidden: #hidden,
                            command: ::std::vec::Vec::from([#( #command ),*]),
                            response: #response
                        }
                    }
                },
            );

            let name = format_ident!("example_{count}");
            count += 1;

            Some(quote! {
                #[test]
                fn #name() {
                    let example = ::dices_man::examples::Example {
                        tags: ::std::vec::Vec::from([#( #tags ),*]),
                        commands: ::std::vec::Vec::from([#( #commands ),*])
                    };
                    crate::check_example_or_panic(&example)
                }
            })
        });

    let nested = page.children().sorted().map(|child| {
        let name = format_ident!("_{}", child.path().last().unwrap());
        let doc = format!(
            " {}. {}",
            child.path().into_iter().format("."),
            child.title()
        );
        let tests = build_tests_for(&child);

        quote! {
            mod #name {
                #![doc=#doc]

                #tests
            }
        }
    });

    quote! {
        #( #tests )*

        #( #nested )*
    }
}
