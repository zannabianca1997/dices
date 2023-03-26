//! CLI commands

use std::{
    fmt::{Display, Write},
    str::FromStr,
};

use either::Either::{Left, Right};
use lazy_regex::regex_captures;
use phf::phf_map;
use rand::Rng;
use strum::EnumDiscriminants;
use termimad::{minimad::TextTemplate, MadSkin};
use thiserror::Error;

use crate::{
    help::{HelpTopic, UnknowTopic},
    parser,
    throws::{Throws, ThrowsError},
};

/// A command for the repl
#[derive(Debug, Clone, EnumDiscriminants)]
#[strum_discriminants(vis(pub), derive(Hash, PartialOrd, Ord))]
pub enum Cmd {
    Throw(Throws),
    Help(HelpTopic),
    Quit,
    None,
}

impl Cmd {
    pub fn execute(self, rng: &mut impl Rng) -> Result<CmdOutput, CmdError> {
        match self {
            Cmd::Throw(throw) => {
                let res = throw.throws(rng)?;

                Ok(CmdOutput::Throw(res))
            }
            Cmd::Help(topic) => Ok(CmdOutput::Help(topic)),
            Cmd::Quit => Ok(CmdOutput::Quit),
            Cmd::None => Ok(CmdOutput::Empty),
        }
    }
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

pub static CMD_STRINGS: phf::Map<&'static str, CmdDiscriminants> = {
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
            .and_then(|(_, cmd, args)| CMD_STRINGS.get(cmd).map(|cmd| (cmd, args)))
            .or((!s.trim().is_empty()).then_some((&CmdDiscriminants::default(), s)))
        {
            cmd.parse_args(args)
                .map_err(|err| ParseCmdError::ParseArgs(*cmd, err))
        } else {
            Ok(Cmd::default())
        }
    }
}

pub enum CmdOutput {
    Throw(Vec<i64>),
    Empty,
    Quit,
    Help(HelpTopic),
}

impl CmdOutput {
    /// Returns `true` if this output is the last of the session
    #[must_use]
    pub fn is_final(&self) -> bool {
        matches!(self, Self::Quit)
    }

    /// Print this output on the screen
    pub fn print(&self, skin: &MadSkin, pretty: bool, interactive: bool) {
        match self {
            CmdOutput::Throw(v) => {
                let arr_str = {
                    let mut buf = String::new();
                    for i in v.into_iter().map(Left).intersperse(Right(())) {
                        match i {
                            Left(v) => write!(buf, "{v}"),
                            Right(_) => write!(buf, " "),
                        }
                        .unwrap()
                    }
                    buf
                };
                if interactive {
                    let text_template = TextTemplate::from(r"**Results:** ${results}");
                    let mut expander = text_template.expander();
                    expander.set("results", &arr_str);
                    skin.print_expander(expander);
                } else {
                    println!("{arr_str}")
                }
            }
            CmdOutput::Help(topic) => topic.print(skin),
            CmdOutput::Empty => (),
            CmdOutput::Quit => {
                if interactive {
                    skin.print_text(if pretty {
                        "\n⛓️ **Bye!** 🐉"
                    } else {
                        "**Bye!**"
                    })
                } else {
                    ()
                }
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum CmdError {
    #[error("Error while parsing command")]
    Parsing(
        #[source]
        #[from]
        ParseCmdError,
    ),
    #[error("Error while evaluating throws")]
    Throwing(
        #[source]
        #[from]
        ThrowsError,
    ),
}
