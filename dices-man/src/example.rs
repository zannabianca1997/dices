//! Stuff to help parsing and making sense of code examples

use std::{ ops::Deref, str::FromStr};

use lazy_regex::{regex_captures, regex_if};
use nunny::NonEmpty;

use dices_ast::{expression::Expression, matcher::Matcher, parse::parse_file, values::ValueNull};

#[derive(Debug,Clone,Hash)]
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
                start.to_owned()
            } else {
                let mut cmd = start.to_owned();
                for line in cont.trim_start().lines() {
                    cmd.push('\n');
                    cmd.push_str(line.trim_start().strip_prefix("...").unwrap())
                }
                cmd
            };
            (
                CodeExampleCommand {
                    ignore: false,
                    command: parse_file(&cmd).expect("Cannot parse command"),
                    src:cmd
                },
                &test[full.len()..],
            )
        }
    )
    .or_else(|| {
        // try to capture an ignored command
        regex_if!(
            r"\A(?<full>[^\S\r\n]*#[^\S\r\n]*>>>(?<start>.*)$(?<cont>(?:(?:\r\n|\n)^[^\S\r\n]*#[^\S\r\n]*\.\.\..*)$)*)"m,
            test,
            {
                let cmd = if cont.is_empty() {
                    start.to_owned()
                } else {
                    let mut cmd = start.to_owned();
                    for line in cont.lines() {
                        cmd.push_str(line.trim_start().strip_prefix('#').unwrap().trim_start().strip_prefix("...").unwrap())
                    }
                    cmd
                };
                (
                    CodeExampleCommand {
                        ignore: true,
                        command: parse_file(&cmd).expect("Cannot parse command"),
                        src: cmd
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

#[derive(Debug,Clone,Hash)]
pub struct CodeExamplePiece {
   pub cmd: CodeExampleCommand,
   pub res: Option<Matcher<!>>,
}

#[derive(Debug,Clone,Hash)]
pub struct CodeExampleCommand {
    /// Do not check the result of this command
    ///
    /// Used to do setup stuff, as it is not printed
   pub ignore: bool,
    /// The actual command
   pub command: Box<NonEmpty<[Expression<!>]>>,
   /// The source code of the command
   pub src: String
}
