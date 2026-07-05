use std::io::stdout;

use crossterm::tty::IsTty;
use serde::{Deserialize, Serialize};

use super::theme::Theme;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Skin {
    /// If opening and closing banners are shown
    #[serde(skip_serializing_if = "is_true")]
    pub banners: bool,
    /// Use non-ascii character
    #[serde(skip_serializing_if = "is_true")]
    pub graphical: bool,
    /// Colorize the output
    #[serde(skip_serializing_if = "is_true")]
    pub color: bool,
    /// Theme chosen
    pub theme: Theme,
}

impl Default for Skin {
    fn default() -> Self {
        let is_tty = stdout().is_tty();
        Self {
            banners: true,
            graphical: is_tty,
            color: is_tty,
            theme: Theme::default(),
        }
    }
}

fn is_true(x: &bool) -> bool {
    *x
}
