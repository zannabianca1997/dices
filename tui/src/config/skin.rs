use ::std::option::Option;
use std::{
    collections::BTreeMap,
    io::{IsTerminal, stdout},
};

use std::str::FromStr;

use clap::builder::{PossibleValue, TypedValueParser};
use pretty::termcolor::{Color, ColorSpec};
use serde::{Deserialize, Serialize};
use strum::Display;

mod color_spec;

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Skins {
    pub ascii: Skin,
    pub plain: Skin,
    pub fancy: Skin,
    #[serde(flatten)]
    pub others: BTreeMap<String, Skin>,
}
impl Default for Skins {
    fn default() -> Self {
        Self {
            ascii: Skin {
                banners: true,
                emoji: false,
                ansi: false,
                elements: Elements::plain(),
            },
            plain: Skin {
                banners: true,
                emoji: false,
                ansi: true,
                elements: Elements::colored(),
            },
            fancy: Skin {
                banners: true,
                emoji: true,
                ansi: true,
                elements: Elements::colored(),
            },
            others: BTreeMap::new(),
        }
    }
}
impl Skins {
    pub fn get<'s>(&'s self, selected: &SelectedSkin) -> Option<&'s Skin> {
        match selected {
            SelectedSkin::Ascii => Some(&self.ascii),
            SelectedSkin::Plain => Some(&self.plain),
            SelectedSkin::Fancy => Some(&self.fancy),
            SelectedSkin::Other(other) => self.others.get(other),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Skin {
    /// If opening and closing banners are shown
    pub banners: bool,
    /// Use emoji in the banners and the prompt
    pub emoji: bool,
    /// Use ansi escape code to colorize the output
    pub ansi: bool,
    /// Styling of elements
    pub elements: Elements,
}

/// Elements to style
#[derive(Debug, Serialize, Deserialize)]
pub struct Elements {
    /// The prompt
    #[serde(with = "color_spec")]
    pub prompts: ColorSpec,
    /// `null` value
    #[serde(with = "color_spec")]
    pub nulls: ColorSpec,
    /// `true` and `false` value
    #[serde(with = "color_spec")]
    pub bools: ColorSpec,
    /// Integer literals
    #[serde(with = "color_spec")]
    pub integers: ColorSpec,
    /// String literals
    #[serde(with = "color_spec")]
    pub strings: ColorSpec,
    /// Punctuators
    #[serde(with = "color_spec")]
    pub punctuators: ColorSpec,
}

impl Elements {
    pub fn colored() -> Self {
        Self {
            prompts: ColorSpec::new()
                .set_fg(Some(Color::Cyan))
                .set_italic(true)
                .clone(),
            nulls: ColorSpec::new()
                .set_fg(Some(Color::White))
                .set_dimmed(true)
                .clone(),
            bools: ColorSpec::new()
                .set_fg(Some(Color::Green))
                .set_bold(true)
                .clone(),
            integers: ColorSpec::new().set_fg(Some(Color::Red)).clone(),
            strings: ColorSpec::new().set_fg(Some(Color::Magenta)).clone(),
            punctuators: ColorSpec::new().set_fg(Some(Color::Yellow)).clone(),
        }
    }
    pub fn plain() -> Self {
        let null_color = ColorSpec::new().set_reset(false).clone();
        Self {
            prompts: null_color.clone(),
            nulls: null_color.clone(),
            bools: null_color.clone(),
            integers: null_color.clone(),
            strings: null_color.clone(),
            punctuators: null_color.clone(),
        }
    }
}

#[derive(Debug, Clone, Display, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SelectedSkin {
    /// Ascii skin
    ///
    /// No color, no emoji
    Ascii,
    /// Plain skin
    ///
    /// Colors are on, emoji are off
    Plain,
    /// Fancy skin
    ///
    /// Full color and emoji
    Fancy,
    /// Other custom skin
    #[serde(untagged)]
    Other(String),
}

impl Default for SelectedSkin {
    fn default() -> Self {
        if stdout().is_terminal() {
            SelectedSkin::Fancy
        } else {
            SelectedSkin::Ascii
        }
    }
}

impl FromStr for SelectedSkin {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_ascii_lowercase().as_str() {
            "ascii" => Self::Ascii,
            "plain" => Self::Plain,
            "fancy" => Self::Fancy,
            _ => Self::Other(s.to_string()),
        })
    }
}

#[derive(Clone)]
pub struct SkinValueParser;

impl TypedValueParser for SkinValueParser {
    type Value = SelectedSkin;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let s = value
            .to_str()
            .ok_or_else(|| clap::Error::new(clap::error::ErrorKind::InvalidUtf8))?;
        s.parse()
            .map_err(|_| clap::Error::new(clap::error::ErrorKind::ValueValidation))
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            vec![
                PossibleValue::new("ascii").help("Ascii skin"),
                PossibleValue::new("plain").help("Plain skin"),
                PossibleValue::new("fancy").help("Fancy skin"),
                PossibleValue::new("(other)").help("Any custom skin name"),
            ]
            .into_iter(),
        ))
    }
}
