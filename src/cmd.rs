//! CLI commands

use std::{error, fmt::Display, str::FromStr};

use lazy_regex::regex_captures;
use phf::phf_map;
use strum::EnumDiscriminants;
use thiserror::Error;

use crate::{
    help::{HelpTopic, UnknowTopic},
    parser,
    throws::Throws,
};

/// A command for the repl
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(vis(pub))]
pub enum Cmd {
    Throw(Throws),
    Help(HelpTopic),
    Quit,
    None,
}

impl Default for CmdDiscriminants {
    fn default() -> Self {
        Self::Throw
    }
}
impl Default for Cmd {
    fn default() -> Self {
        Self::None
    }
}

impl Display for CmdDiscriminants {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CmdDiscriminants::*;
        write!(
            f,
            "{}",
            match self {
                Throw => "throw",
                Help => "help",
                Quit => "quit",
                None => "(none)",
            }
        )
    }
}

pub static CmdStrings: phf::Map<&'static str, CmdDiscriminants> = {
    use CmdDiscriminants::*;
    phf_map! {
        "throw" => Throw,
        "t" => Throw,
        "help" =>  Help,
        "?"=>Help,
        "quit" => Quit,
        "q" => Quit
    }
};

#[derive(Debug, Error)]
pub enum ParseArgsError {
    #[error("Unexpected argument to {0}: {1:?}")]
    UnexpectedArgument(CmdDiscriminants, String),
    #[error(transparent)]
    UnknowHelpTopic(#[from] UnknowTopic),
    #[error("Cannot parse throws")]
    ThrowsExpr(
        #[from]
        #[source]
        parser::Error,
    ),
}
#[derive(Debug, Error)]
pub enum ParseCmdError {
    #[error("Error during parsing args for {0}")]
    ParseArgs(CmdDiscriminants, #[source] ParseArgsError),
}

impl CmdDiscriminants {
    fn parse_args(self, args: &str) -> Result<Cmd, ParseArgsError> {
        use Cmd::*;
        match self {
            CmdDiscriminants::Throw => Ok(Throw(args.parse()?)),
            CmdDiscriminants::Help => {
                let topic = (!args.trim().is_empty())
                    .then(|| args.trim().parse())
                    .transpose()?
                    .unwrap_or_default();
                Ok(Help(topic))
            }
            CmdDiscriminants::Quit => {
                if !args.trim().is_empty() {
                    Err(ParseArgsError::UnexpectedArgument(self, args.to_owned()))
                } else {
                    Ok(Quit)
                }
            }
            CmdDiscriminants::None => {
                if !args.trim().is_empty() {
                    Err(ParseArgsError::UnexpectedArgument(self, args.to_owned()))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl FromStr for Cmd {
    type Err = ParseCmdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((cmd, args)) = regex_captures!(r"^\s*([a-zA-Z\?]+)(?:\s+(.*))?$", s)
            .and_then(|(_, cmd, args)| CmdStrings.get(cmd).map(|cmd| (cmd, args)))
            .or((!s.trim().is_empty()).then_some((&CmdDiscriminants::default(), s)))
        {
            cmd.parse_args(args)
                .map_err(|err| ParseCmdError::ParseArgs(*cmd, err))
        } else {
            Ok(Cmd::default())
        }
    }
}
