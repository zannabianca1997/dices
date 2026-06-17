//! Deserializing and serializing of color specs

use pretty::termcolor::{Color, ColorSpec};
use serde::{Deserialize, Serialize, de};

pub fn deserialize<'de, D>(deserializer: D) -> Result<ColorSpec, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let ColorSpecStruct {
        fg,
        bg,
        bold,
        intense,
        underline,
        dimmed,
        italic,
        reset,
        strikethrough,
    } = ColorSpecStruct::deserialize(deserializer)?;
    let mut color_spec = ColorSpec::new();
    color_spec
        .set_bg(bg)
        .set_fg(fg)
        .set_bold(bold)
        .set_intense(intense)
        .set_underline(underline)
        .set_dimmed(dimmed)
        .set_italic(italic)
        .set_reset(reset)
        .set_strikethrough(strikethrough);
    Ok(color_spec)
}

pub fn serialize<S>(spec: &ColorSpec, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let spec_struct = ColorSpecStruct {
        fg: spec.fg().copied(),
        bg: spec.bg().copied(),
        bold: spec.bold(),
        intense: spec.intense(),
        underline: spec.underline(),
        dimmed: spec.dimmed(),
        italic: spec.italic(),
        reset: spec.reset(),
        strikethrough: spec.strikethrough(),
    };
    spec_struct.serialize(serializer)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
struct ColorSpecStruct {
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    fg: Option<Color>,
    #[serde(
        deserialize_with = "deserialize_color",
        serialize_with = "serialize_color"
    )]
    bg: Option<Color>,
    #[serde(skip_serializing_if = "is_false")]
    bold: bool,
    #[serde(skip_serializing_if = "is_false")]
    intense: bool,
    #[serde(skip_serializing_if = "is_false")]
    underline: bool,
    #[serde(skip_serializing_if = "is_false")]
    dimmed: bool,
    #[serde(skip_serializing_if = "is_false")]
    italic: bool,
    #[serde(skip_serializing_if = "is_true")]
    reset: bool,
    #[serde(skip_serializing_if = "is_false")]
    strikethrough: bool,
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn is_true(b: &bool) -> bool {
    *b
}

impl Default for ColorSpecStruct {
    fn default() -> Self {
        Self {
            fg: None,
            bg: None,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum ColorSerDe {
    Ansi(u8),
    Str(String),
}

impl From<Color> for ColorSerDe {
    fn from(c: Color) -> Self {
        match c {
            Color::Black => Self::Str("black".into()),
            Color::Blue => Self::Str("blue".into()),
            Color::Green => Self::Str("green".into()),
            Color::Red => Self::Str("red".into()),
            Color::Cyan => Self::Str("cyan".into()),
            Color::Magenta => Self::Str("magenta".into()),
            Color::Yellow => Self::Str("yellow".into()),
            Color::White => Self::Str("white".into()),
            Color::Rgb(r, g, b) => Self::Str(format!("#{r:02x}{g:02x}{b:02x}")),
            Color::Ansi256(n) => Self::Ansi(n),
            _ => Self::Str("unknown".into()),
        }
    }
}

impl TryFrom<ColorSerDe> for Color {
    type Error = String;

    fn try_from(value: ColorSerDe) -> Result<Self, Self::Error> {
        match value {
            ColorSerDe::Ansi(n) => Ok(Color::Ansi256(n)),
            ColorSerDe::Str(s) => {
                let s = s.trim();
                if let Some(code) = s.strip_prefix('#') {
                    if code.len() != 6 || code.chars().any(|ch| !ch.is_ascii_hexdigit()) {
                        Err("Expected hex color code in the form #HHHHHH".into())
                    } else {
                        let r = u8::from_str_radix(&code[0..2], 16).unwrap();
                        let g = u8::from_str_radix(&code[2..4], 16).unwrap();
                        let b = u8::from_str_radix(&code[4..6], 16).unwrap();
                        Ok(Color::Rgb(r, g, b))
                    }
                } else {
                    match s.to_ascii_lowercase().as_str() {
                        "black" => Ok(Color::Black),
                        "blue" => Ok(Color::Blue),
                        "green" => Ok(Color::Green),
                        "red" => Ok(Color::Red),
                        "cyan" => Ok(Color::Cyan),
                        "magenta" => Ok(Color::Magenta),
                        "yellow" => Ok(Color::Yellow),
                        "white" => Ok(Color::White),
                        _ => Err(format!(
                            "Expected either a color name, hex color code, or Ansi256 index, got: {s}"
                        )),
                    }
                }
            }
        }
    }
}

fn serialize_color<S>(c: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match c {
        None => serializer.serialize_none(),
        Some(c) => ColorSerDe::from(*c).serialize(serializer),
    }
}

fn deserialize_color<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let Some(v) = <Option<ColorSerDe>>::deserialize(deserializer)? else {
        return Ok(None);
    };
    Ok(Some(Color::try_from(v).map_err(de::Error::custom)?))
}
