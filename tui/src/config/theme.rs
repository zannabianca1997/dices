use std::borrow::Cow;

use dices_print::theme::{Color as ThemeColor, Style};
use elsa::sync::FrozenMap;
use pretty::termcolor::{Color as TermColor, ColorSpec};
use rust_embed::Embed;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use yoke::Yoke;

use dices_print::{Element, PromptElement, theme::Theme as StyleSheet};

/// Themes bundled into the binary
#[derive(Embed)]
#[folder = "$CARGO_MANIFEST_DIR/themes"]
struct Themes;

pub fn available_themes() -> Vec<String> {
    let mut names: Vec<String> = Themes::iter()
        .filter_map(|f| {
            f.ends_with(".css")
                .then(|| f[..f.len() - ".css".len()].to_owned())
        })
        .collect();

    if let Some(dir) = super::themes_dir()
        && let Ok(entries) = std::fs::read_dir(&dir)
    {
        for entry in entries.flatten().filter_map(|e| {
            let path = e.path();
            path.extension()
                .is_some_and(|e| e == "css")
                .then(|| path.file_stem())
                .flatten()
                .map(|s| s.to_string_lossy().into_owned())
        }) {
            if !names.contains(&entry) {
                names.push(entry);
            }
        }
    }

    names
}

const DEFAULT_TRUE_COLORS: &str = "CatppuccinMocha";
const DEFAULT_LOW_COLORS: &str = "LowColor";

pub struct Theme {
    name: String,
    sheet: Yoke<StyleSheet<'static>, Vec<Cow<'static, str>>>,
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
        Self {
            name: if colors > 256 {
                DEFAULT_TRUE_COLORS.to_owned()
            } else {
                DEFAULT_LOW_COLORS.to_owned()
            },
            sheet: Yoke::attach_to_cart(Vec::new(), |_| Default::default()),
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

        let name =
            Option::<String>::deserialize(deserializer)?.unwrap_or_else(|| "Default".to_owned());

        let mut cart: Vec<Cow<'static, str>> = Vec::with_capacity(2);

        let has_embedded = if let Some(embedded) = Themes::get(&format!("{name}.css")) {
            let embedded_css = String::from_utf8(embedded.data.into_owned()).map_err(|err| {
                D::Error::custom(format!("could not read embedded theme `{name}`: {err}"))
            })?;
            cart.push(Cow::Owned(embedded_css));
            true
        } else {
            false
        };

        let has_custom = if let Some(path) = super::themes_dir() {
            if let Ok(custom_css) = std::fs::read_to_string(path.join(&name).with_extension("css"))
            {
                cart.push(Cow::Owned(custom_css));
                true
            } else {
                false
            }
        } else {
            false
        };

        if !has_embedded && !has_custom {
            return Err(D::Error::custom(format!("theme `{name}` not found")));
        }

        let sheet = Yoke::try_attach_to_cart(cart, |css_vec| {
            let sheet = if has_embedded {
                let mut s = StyleSheet::parse(&css_vec[0]);
                if css_vec.len() > 1 {
                    s.parse_more(&css_vec[1]);
                }
                s
            } else {
                StyleSheet::parse(&css_vec[0])
            };
            Ok::<_, D::Error>(sheet)
        })?;

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

fn reedline_color(spec: Style) -> reedline::Color {
    spec.fg_color
        .map(|c| term_to_reedline(&convert_color(c)))
        .unwrap_or(reedline::Color::Reset)
}

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
