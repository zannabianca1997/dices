use std::{borrow::Cow, str::FromStr};

use anyhow::Context;
use dices_ast::{expression::Expression, matcher::Matcher, parse::parse_file, values::ValueNull};
use dices_engine::solve::Engine;
use lazy_regex::{regex_find, regex_if};
use nunny::NonEmpty;
use rand::rngs::SmallRng;

/// Main testing function
pub(crate) fn test_inner(test: &str, tags: &[&str]) {
    // Parse the test
    let test: DocTest = test.parse().expect("The test should be parseable");
    // Create the engine
    let mut engine: Engine<SmallRng> = Engine::new();
    // run the test
    for (n, piece) in (&*test.0).into_iter().enumerate() {
        let res = engine.eval_multiple(&piece.cmd.command).expect("Error in the execution of the doctest!");
        if let Some(checker) = piece.res.as_ref() {
            assert!(checker.is_match(&res), "The result number {} was {}, not satisfing the matcher", n+1, res)
        }
    }
}

struct DocTest(Box<[DocTestPiece]>);

impl FromStr for DocTest {
    type Err = !;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
         // parse the test
    let mut items = vec![];
    let mut test = s.trim_start();

    while let Some((cmd, rest)) = regex_if!(
        r"\A(?<full>[^\S\r\n]*>>>(?<start>.*)$(?<cont>(?:(?:\r\n|\n)^[^\S\r\n]*\.\.\..*)$)*)"m,
        test,
        {
            let cmd = if cont.is_empty() {
                Cow::Borrowed(start)
            } else {
                let mut cmd = start.to_owned();
                for line in cont.lines() {
                    cmd.push_str(line.trim_start().strip_prefix("...").unwrap())
                }
                Cow::Owned(cmd)
            };
            (
                DocTestCommand {
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
                    DocTestCommand {
                        ignore: true,
                        command: parse_file(&cmd).context("Cannot parse command").unwrap(),
                    },
                    &test[full.len()..],
                )
            }
        )
    }) {
        // need now to split the result
        let res = regex_find!(r"\A((?:.|\n)*?)(?:^[^\S\r\n]*(?:#[^\S\r\n]*)?>>>|\z)",rest).expect("The regex is infallible");
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
        items.push(DocTestPiece {cmd, res})
    }
    assert_eq!(test.trim(), "", "Cannot recognize command prompt");

    Ok(Self(items.into_boxed_slice()))
    }
}

struct DocTestPiece {
    cmd: DocTestCommand,
    res: Option<Matcher>,
}

struct DocTestCommand {
    /// Do not check the result of this command
    ///
    /// Used to do setup stuff, as it is not printed
    ignore: bool,
    /// The actual command
    command: Box<NonEmpty<[Expression]>>,
}

