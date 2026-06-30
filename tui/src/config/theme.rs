use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
};

use dices_print::theme::{Color as ThemeColor, Style};
use elsa::sync::FrozenMap;
use pretty::termcolor::{Color as TermColor, ColorSpec};
use rust_embed::Embed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yoke::Yoke;

use dices_print::{Element, PromptElement, theme::Theme as StyleSheet};

/// Themes bundled into the binary, written to disk on first run.
#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/themes"]
struct Themes;

const DEFAULT_TRUE_COLORS: &str = "CatppuccinMocha";
const DEFAULT_LOW_COLORS: &str = "LowColor";

/// Writes any bundled theme not already present in `themes_dir` (idempotent),
/// mirroring `write_config_file_if_not_exists`.
pub fn write_themes_if_not_exists(themes_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(themes_dir)?;
    for name in Themes::iter() {
        let path = themes_dir.join(&*name);
        match File::create_new(&path) {
            Ok(mut file) => file.write_all(&Themes::get(&name).unwrap().data)?,
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

pub struct Theme {
    name: String,
    sheet: Yoke<StyleSheet<'static>, String>,
    cache: FrozenMap<Element, Box<ColorSpec>>,
    prompt: reedline::Color,
    prompt_indicator: reedline::Color,
    prompt_multiline: nu_ansi_term::Color,
    prompt_right: reedline::Color,
}

impl std::fmt::Debug for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Theme")
            .field("name", &self.name)
            .field("sheet", &self.sheet)
            .field("prompt", &self.prompt)
            .field("prompt_indicator", &self.prompt_indicator)
            .field("prompt_multiline", &self.prompt_multiline)
            .field("prompt_right", &self.prompt_right)
            .finish()
    }
}

impl Theme {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn style(&self, element: Element) -> &ColorSpec {
        self.cache.get(&element).unwrap_or_else(|| {
            self.cache
                .insert(element, Box::new(color_spec(style(&self.sheet, element))))
        })
    }

    pub fn prompt(&self) -> reedline::Color {
        self.prompt
    }

    pub fn prompt_indicator(&self) -> reedline::Color {
        self.prompt_indicator
    }

    pub fn prompt_multiline(&self) -> nu_ansi_term::Color {
        self.prompt_multiline
    }

    pub fn prompt_right(&self) -> reedline::Color {
        self.prompt_right
    }
}

impl Default for Theme {
    fn default() -> Self {
        let colors = crossterm::style::available_color_count();
        // Only `name` is persisted (see `Serialize`), so the resolved colours
        // are placeholders; the named theme is loaded on the next deserialize.
        Self {
            name: if colors > 256 {
                DEFAULT_TRUE_COLORS.to_owned()
            } else {
                DEFAULT_LOW_COLORS.to_owned()
            },
            sheet: Yoke::attach_to_cart(String::new(), |_| Default::default()),
            cache: Default::default(),
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

        // `<config>/themes/<name>.css`.
        let path = super::themes_dir()
            .ok_or_else(|| D::Error::custom("could not determine the configuration directory"))?
            .join(&name)
            .with_extension("css");

        let css = std::fs::read_to_string(&path).map_err(|err| {
            D::Error::custom(format!(
                "could not read theme `{name}` at {}: {err}",
                path.display()
            ))
        })?;
        let sheet = Yoke::attach_to_cart(css, |raw| StyleSheet::parse(raw));

        // Map the serializable spec onto the real `ColorSpec`, then read the
        // prompt colors back out of the resolved theme.
        let prompt = reedline_color(style(&sheet, Element::Prompt(None)));
        let prompt_indicator = reedline_color(style(
            &sheet,
            Element::Prompt(Some(PromptElement::Indicator)),
        ));
        let prompt_multiline = nu_color(style(
            &sheet,
            Element::Prompt(Some(PromptElement::Multiline)),
        ));
        let prompt_right =
            reedline_color(style(&sheet, Element::Prompt(Some(PromptElement::Right))));

        Ok(Self {
            name,
            sheet,
            prompt,
            cache: Default::default(),
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

fn style<C>(sheet: &Yoke<StyleSheet<'static>, C>, element: Element) -> Style {
    sheet.get().style(element)
}

fn convert_color(c: ThemeColor) -> TermColor {
    match c {
        ThemeColor::Black => TermColor::Black,
        ThemeColor::Blue => TermColor::Blue,
        ThemeColor::Green => TermColor::Green,
        ThemeColor::Red => TermColor::Red,
        ThemeColor::Cyan => TermColor::Cyan,
        ThemeColor::Magenta => TermColor::Magenta,
        ThemeColor::Yellow => TermColor::Yellow,
        ThemeColor::White => TermColor::White,
        ThemeColor::Ansi256(n) => TermColor::Ansi256(n),
        ThemeColor::Rgb(r, g, b) => TermColor::Rgb(r, g, b),
    }
}

fn color_spec(spec: Style) -> ColorSpec {
    let mut cs = ColorSpec::new();
    if let Some(fg) = spec.fg_color {
        cs.set_fg(Some(convert_color(fg)));
    }
    if let Some(bg) = spec.bg_color {
        cs.set_bg(Some(convert_color(bg)));
    }
    cs.set_bold(spec.bold);
    cs.set_intense(spec.intense);
    cs.set_underline(spec.underline);
    cs.set_dimmed(spec.dimmed);
    cs.set_italic(spec.italic);
    cs.set_reset(spec.reset);
    cs
}

/// Foreground of a resolved [`Style`] as a [`reedline::Color`], defaulting
/// to the terminal's reset color when unset.
fn reedline_color(spec: Style) -> reedline::Color {
    spec.fg_color
        .map(|c| term_to_reedline(&convert_color(c)))
        .unwrap_or(reedline::Color::Reset)
}

/// Foreground of a resolved [`Style`] as a [`nu_ansi_term::Color`],
/// defaulting to the terminal's default color when unset.
fn nu_color(spec: Style) -> nu_ansi_term::Color {
    spec.fg_color
        .map(|c| term_to_nu(&convert_color(c)))
        .unwrap_or(nu_ansi_term::Color::Default)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_themes_is_idempotent() {
        let dir = std::env::temp_dir().join(format!("dices-themes-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        write_themes_if_not_exists(&dir).expect("first write");
        write_themes_if_not_exists(&dir).expect("second write is a no-op");

        for name in Themes::iter() {
            assert!(dir.join(&*name).is_file(), "{name} written");
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
