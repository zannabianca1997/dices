//! Help command

use std::str::FromStr;

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
    pub fn help(&self) -> &'static str {
        match self {
            HelpTopic::General => &general::HELP,
            _ => unimplemented!("Missing help for topic {self:?}"),
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
