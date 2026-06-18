use std::io::{IsTerminal, stdout};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Skin {
    /// If opening and closing banners are shown
    pub banners: bool,
    /// Use non-ascii character
    pub graphical: bool,
    /// Colorize the output
    pub color: bool,
}

impl Default for Skin {
    fn default() -> Self {
        let is_tty = stdout().is_terminal();
        Self {
            banners: true,
            graphical: is_tty,
            color: is_tty,
        }
    }
}
