use std::fs::{self, File};
use std::io::{self, Write};
use std::path::Path;

use dices_print::{Annotation, PromptElement};
use pretty::termcolor::{Color as TermColor, ColorSpec};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Themes bundled into the binary, written to disk on first run.
static BUNDLED: &[(&str, &str)] = &[
    (
        "CatppuccinLatte",
        include_str!("themes/CatppuccinLatte.toml"),
    ),
    (
        "CatppuccinMocha",
        include_str!("themes/CatppuccinMocha.toml"),
    ),
];
const DEFAULT: &str = "CatppuccinMocha";

/// Writes any bundled theme not already present in `themes_dir` (idempotent),
/// mirroring `write_config_file_if_not_exists`.
pub fn write_themes_if_not_exists(themes_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(themes_dir)?;
    for (name, content) in BUNDLED {
        let path = themes_dir.join(format!("{name}.toml"));
        match File::create_new(&path) {
            Ok(mut file) => file.write_all(content.as_bytes())?,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct Theme {
    pub name: String,
    pub content: dices_print::theme::Theme<ColorSpec>,
    pub prompt: reedline::Color,
    pub prompt_indicator: reedline::Color,
    pub prompt_multiline: nu_ansi_term::Color,
    pub prompt_right: reedline::Color,
}

impl Default for Theme {
    fn default() -> Self {
        // Only `name` is persisted (see `Serialize`), so the resolved colours
        // are placeholders; the named theme is loaded on the next deserialize.
        Self {
            name: DEFAULT.to_owned(),
            content: dices_print::theme::Theme::default(),
            prompt: reedline::Color::Reset,
            prompt_indicator: reedline::Color::Reset,
            prompt_multiline: nu_ansi_term::Color::Default,
            prompt_right: reedline::Color::Reset,
        }
    }
}

impl<'de> Deserialize<'de> for Theme {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        // The skin only stores the theme *name*; the data lives in a file.
        let name =
            Option::<String>::deserialize(deserializer)?.unwrap_or_else(|| "Default".to_owned());

        // `<config>/themes/<name>.toml`.
        let path = super::directories()
            .ok_or_else(|| D::Error::custom("could not determine the configuration directory"))?
            .config_dir()
            .join("themes")
            .join(&name)
            .with_extension("toml");

        let raw = std::fs::read_to_string(&path).map_err(|err| {
            D::Error::custom(format!(
                "could not read theme `{name}` at {}: {err}",
                path.display()
            ))
        })?;
        let content: dices_print::theme::Theme<SerDeColorSpec> = toml::from_str(&raw)
            .map_err(|err| D::Error::custom(format!("could not parse theme `{name}`: {err}")))?;

        // Map the serializable spec onto the real `ColorSpec`, then read the
        // prompt colors back out of the resolved theme.
        let content = content.map(ColorSpec::from);
        let prompt = reedline_color(content.style(Annotation::Prompt(None)));
        let prompt_indicator =
            reedline_color(content.style(Annotation::Prompt(Some(PromptElement::Indicator))));
        let prompt_multiline =
            nu_color(content.style(Annotation::Prompt(Some(PromptElement::Multiline))));
        let prompt_right =
            reedline_color(content.style(Annotation::Prompt(Some(PromptElement::Right))));

        Ok(Self {
            name,
            content,
            prompt,
            prompt_indicator,
            prompt_multiline,
            prompt_right,
        })
    }
}

impl Serialize for Theme {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // The skin only persists the theme name (round-tripping `Deserialize`).
        Some(&self.name).serialize(serializer)
    }
}

/// Foreground of a resolved [`ColorSpec`] as a [`reedline::Color`], defaulting
/// to the terminal's reset color when unset.
fn reedline_color(spec: &ColorSpec) -> reedline::Color {
    spec.fg().map_or(reedline::Color::Reset, term_to_reedline)
}

/// Foreground of a resolved [`ColorSpec`] as a [`nu_ansi_term::Color`],
/// defaulting to the terminal's default color when unset.
fn nu_color(spec: &ColorSpec) -> nu_ansi_term::Color {
    spec.fg().map_or(nu_ansi_term::Color::Default, term_to_nu)
}

fn term_to_reedline(color: &TermColor) -> reedline::Color {
    use reedline::Color as C;
    match *color {
        TermColor::Black => C::Black,
        TermColor::Blue => C::Blue,
        TermColor::Green => C::Green,
        TermColor::Red => C::Red,
        TermColor::Cyan => C::Cyan,
        TermColor::Magenta => C::Magenta,
        TermColor::Yellow => C::Yellow,
        TermColor::White => C::White,
        TermColor::Ansi256(n) => C::AnsiValue(n),
        TermColor::Rgb(r, g, b) => C::Rgb { r, g, b },
        _ => C::Reset,
    }
}

fn term_to_nu(color: &TermColor) -> nu_ansi_term::Color {
    use nu_ansi_term::Color as C;
    match *color {
        TermColor::Black => C::Black,
        TermColor::Blue => C::Blue,
        TermColor::Green => C::Green,
        TermColor::Red => C::Red,
        TermColor::Cyan => C::Cyan,
        TermColor::Magenta => C::Magenta,
        TermColor::Yellow => C::Yellow,
        TermColor::White => C::White,
        TermColor::Ansi256(n) => C::Fixed(n),
        TermColor::Rgb(r, g, b) => C::Rgb(r, g, b),
        _ => C::Default,
    }
}

#[derive(Debug, Deserialize)]
struct SerDeColorSpec {
    fg_color: Option<SerDeColor>,
    bg_color: Option<SerDeColor>,
    bold: bool,
    intense: bool,
    underline: bool,
    dimmed: bool,
    italic: bool,
    reset: bool,
    strikethrough: bool,
}

impl Default for SerDeColorSpec {
    fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
            bold: false,
            intense: false,
            underline: false,
            dimmed: false,
            italic: false,
            reset: true,
            strikethrough: false,
        }
    }
}

impl From<SerDeColorSpec> for ColorSpec {
    fn from(spec: SerDeColorSpec) -> Self {
        let mut out = ColorSpec::new();
        out.set_fg(spec.fg_color.map(|c| c.0));
        out.set_bg(spec.bg_color.map(|c| c.0));
        out.set_bold(spec.bold);
        out.set_intense(spec.intense);
        out.set_underline(spec.underline);
        out.set_dimmed(spec.dimmed);
        out.set_italic(spec.italic);
        out.set_reset(spec.reset);
        out.set_strikethrough(spec.strikethrough);
        out
    }
}

/// A color as written in a theme file, parsed on deserialization. Accepts a
/// raw ansi256 integer, a `#RRGGBB` 24-bit hex string, or any color string
/// understood by `termcolor` (names, ansi256, `r,g,b` triples). A malformed
/// color is a hard error rather than being silently dropped.
#[derive(Debug)]
struct SerDeColor(TermColor);

impl<'de> Deserialize<'de> for SerDeColor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            String(String),
            Int(u8),
        }

        let color = match Raw::deserialize(deserializer)? {
            Raw::Int(value) => TermColor::Ansi256(value),
            Raw::String(s) if s.starts_with('#') => {
                parse_hex(&s).map_err(serde::de::Error::custom)?
            }
            Raw::String(s) => s.parse().map_err(serde::de::Error::custom)?,
        };
        Ok(SerDeColor(color))
    }
}

/// Parses a `#RRGGBB` 24-bit hex color into [`TermColor::Rgb`].
fn parse_hex(s: &str) -> Result<TermColor, String> {
    let hex = &s[1..];
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!("`{s}` is not a 24-bit `#RRGGBB` hex color"));
    }
    let byte = |range: std::ops::Range<usize>| {
        u8::from_str_radix(&hex[range], 16).expect("validated as hex above")
    };
    Ok(TermColor::Rgb(byte(0..2), byte(2..4), byte(4..6)))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every bundled theme must parse and resolve against the deserializer's
    /// path/`SerDeColorSpec` schema, guarding the embedded TOML from drift.
    #[test]
    fn bundled_themes_are_valid() {
        for (name, content) in BUNDLED {
            toml::from_str::<dices_print::theme::Theme<SerDeColorSpec>>(content)
                .unwrap_or_else(|err| panic!("bundled theme `{name}` is invalid: {err}"));
        }
    }

    #[test]
    fn write_themes_is_idempotent() {
        let dir = std::env::temp_dir().join(format!("dices-themes-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        write_themes_if_not_exists(&dir).expect("first write");
        write_themes_if_not_exists(&dir).expect("second write is a no-op");

        for (name, _) in BUNDLED {
            assert!(dir.join(format!("{name}.toml")).is_file(), "{name} written");
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
