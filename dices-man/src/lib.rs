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

pub mod example {
    //! Stuff to help parsing and making sense of code examples
    
    use std::{borrow::Cow, ops::Deref, str::FromStr};

    use anyhow::Context;
    use lazy_regex::{regex_captures, regex_if};
    use nunny::NonEmpty;
    
    use dices_ast::{expression::Expression, matcher::Matcher, parse::parse_file, values::ValueNull};

    #[derive(Debug,Clone)]
    pub struct CodeExample(Box<[CodeExamplePiece]>);
    
    impl Deref for CodeExample {
        type Target = [CodeExamplePiece];
    
        fn deref(&self) -> &Self::Target {
           &*self.0
        }
    }
    
    impl FromStr for CodeExample {
        type Err = !;
    
        fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
             // parse the test
        let mut items = vec![];
        let mut test = s.trim_start();
    
        while let Some((cmd, rest)) = regex_if!(
            r"\A(?<full>[^\S\r\n]*>>>(?<start>.*)$(?<cont>(?:(?:\r\n|\n)^[^\S\r\n]*\.\.\..*$)*))"m,
            test,
            {
                let cmd = if cont.is_empty() {
                    Cow::Borrowed(start)
                } else {
                    let mut cmd = start.to_owned();
                    for line in cont.trim_start().lines() {
                        cmd.push_str(line.trim_start().strip_prefix("...").unwrap())
                    }
                    Cow::Owned(cmd)
                };
                (
                    CodeExampleCommand {
                        ignore: false,
                        command: parse_file(&cmd).context("Cannot parse command").unwrap(),
                    },
                    &test[full.len()..],
                )
            }
        )
        .or_else(|| {
            // try to capture an ignored block
            regex_if!(
                r"\A(?<full>[^\S\r\n]*#[^\S\r\n]*>>>(?<start>.*)$(?<cont>(?:(?:\r\n|\n)^[^\S\r\n]*#[^\S\r\n]*\.\.\..*)$)*)"m,
                test,
                {
                    let cmd = if cont.is_empty() {
                        Cow::Borrowed(start)
                    } else {
                        let mut cmd = start.to_owned();
                        for line in cont.lines() {
                            cmd.push_str(line.trim_start().strip_prefix('#').unwrap().trim_start().strip_prefix("...").unwrap())
                        }
                        Cow::Owned(cmd)
                    };
                    (
                        CodeExampleCommand {
                            ignore: true,
                            command: parse_file(&cmd).context("Cannot parse command").unwrap(),
                        },
                        &test[full.len()..],
                    )
                }
            )
        }) {
            // need now to split the result
            let (_,res) = regex_captures!(r"\A((?:.|\n)*?)(?:^[^\S\r\n]*(?:#[^\S\r\n]*)?>>>|\z)"m,rest).expect("The regex is infallible");
            test = &rest[res.len()..];
            // conversting the result
            let res = res.trim();
            let res = if res.starts_with('#') {
                for l in res
                .lines()
                .filter(|l| 
                    !l
                    .trim()
                    .is_empty()
                ) {
                    assert!(l.trim_start().starts_with('#'), "Inconsistent ignoring of result lines")
                }
                None
            } else {
                if res == "" {
                    // empty result corresponds to empty values
                    Some(Matcher::Exact(ValueNull.into()))
                } else {
                    Some(res.parse().expect("The value must be a valid result matcher"))
                }
            };
            items.push(CodeExamplePiece {cmd, res})
        }
        assert_eq!(test.trim(), "", "Cannot recognize command prompt");
    
        Ok(Self(items.into_boxed_slice()))
        }
    }
    
    #[derive(Debug,Clone)]
    pub struct CodeExamplePiece {
       pub cmd: CodeExampleCommand,
       pub res: Option<Matcher>,
    }
    
    #[derive(Debug,Clone)]
    pub struct CodeExampleCommand {
        /// Do not check the result of this command
        ///
        /// Used to do setup stuff, as it is not printed
       pub ignore: bool,
        /// The actual command
       pub command: Box<NonEmpty<[Expression]>>,
    }
    }