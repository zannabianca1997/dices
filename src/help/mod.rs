//! Help command

use std::{fmt::Debug, str::FromStr};

use termimad::{terminal_size, FmtText, MadSkin};
use thiserror::Error;

use crate::cmd::{CmdDiscriminants, CMD_STRINGS};
mod general;

#[derive(Debug, Clone, Copy, Default)]
pub enum HelpTopic {
    #[default]
    General,
    Cmd(CmdDiscriminants),
}

impl HelpTopic {
    pub fn print(self, skin: &MadSkin) {
        use CmdDiscriminants::*;
        use HelpTopic::*;
        match self {
            General => {
                let formatted = FmtText::from_text(
                    skin,
                    general::HELP.clone(),
                    Some(terminal_size().0 as usize),
                );
                print!("{formatted}")
            }
            Cmd(discr) => skin.print_text(match discr {
                Throw => include_str!("throw.md"),
                Help => include_str!("help.md"),
                Quit => include_str!("quit.md"),
                None => unreachable!("None shouln't be parseable for `help`, not having a command"),
            }),
        }
    }
}

#[derive(Debug, Error)]
#[error("Unrecognized help topic {0:?}")]
pub struct UnknowTopic(String);

impl FromStr for HelpTopic {
    type Err = UnknowTopic;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Ok(Self::default())
        } else if let Some(cmd) = CMD_STRINGS.get(&s.to_lowercase()) {
            Ok(Self::Cmd(*cmd))
        } else {
            Err(UnknowTopic(s.to_owned()))
        }
    }
}
