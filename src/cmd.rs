//! CLI commands

use std::str::FromStr;

use pest::Parser;

use crate::{help::HelpTopic, parser, throws::Throws};

/// A command for the repl
#[derive(Debug, Clone)]
pub enum Cmd {
    Throws(Throws),
    Throw(Throws),
    Help(HelpTopic),
    Quit,
    None,
}

impl FromStr for Cmd {
    type Err = pest::error::Error<parser::Rule>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parser::ThrowsParser::parse(parser::Rule::cmd, s)
            .map(|mut pairs| pairs.next().unwrap().into())
    }
}
