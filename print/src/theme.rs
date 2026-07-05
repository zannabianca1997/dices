use std::io::Write;

use optional_struct::{Applicable, optional_struct};
use simplecss::{AttributeOperator, Element as CssElement, PseudoClass, StyleSheet};
use yoke::Yokeable;

use crate::{
    AstElement, DelimiterKind, Element, ErrorElement, List, ListStyle, MarkdownElement,
    PromptElement, ValueElement,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Black,
    Blue,
    Green,
    Red,
    Cyan,
    Magenta,
    Yellow,
    White,
    Ansi256(u8),
    Rgb(u8, u8, u8),
}

#[derive(Debug, Clone)]
#[optional_struct]
pub struct Style {
    pub fg_color: Option<Color>,
    pub bg_color: Option<Color>,
    pub bold: bool,
    pub intense: bool,
    pub underline: bool,
    pub dimmed: bool,
    pub italic: bool,
    pub reset: bool,
    pub strikethrough: bool,
}

impl Default for Style {
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

#[derive(Debug, Yokeable)]
pub struct Theme<'a> {
    stylesheet: StyleSheet<'a>,
}

impl<'a> Theme<'a> {
    /// Parse a css theme
    pub fn parse(css: &'a str) -> Self {
        Self {
            stylesheet: StyleSheet::parse(css),
        }
    }

    /// Add more rules to the theme.
    ///
    /// The new rules will override the current ones if the specificity is the
    /// same.
    pub fn parse_more(&mut self, css: &'a str) {
        self.stylesheet.parse_more(css);
    }

    pub fn style(&self, annotation: &Element) -> Style {
        // all styles cascade, so take the style of the parent
        let mut style = annotation
            .parent_element()
            .map_or_else(Default::default, |parent| self.style(&parent));

        for rule in &self.stylesheet.rules {
            if rule.selector.matches(annotation) {
                for decl in &rule.declarations {
                    OptionalStyle::from_declaration(decl.name, decl.value).apply_to(&mut style);
                }
            }
        }

        style
    }
}

impl Default for Theme<'_> {
    fn default() -> Self {
        Self {
            stylesheet: StyleSheet::default(),
        }
    }
}

impl OptionalStyle {
    fn from_declaration(name: &str, value: &str) -> Self {
        let value = value.trim();
        match name {
            "color" => Self {
                fg_color: parse_color(value),
                ..Default::default()
            },
            "background-color" => Self {
                bg_color: parse_color(value),
                ..Default::default()
            },
            "font-weight" if value == "bold" => Self {
                bold: Some(true),
                ..Default::default()
            },
            "font-style" if value == "italic" => Self {
                italic: Some(true),
                ..Default::default()
            },
            "text-decoration" => {
                let parts: Vec<&str> = value.split_whitespace().collect();
                Self {
                    underline: if parts.contains(&"underline") {
                        Some(true)
                    } else if parts.contains(&"none") {
                        Some(false)
                    } else {
                        None
                    },
                    strikethrough: if parts.contains(&"line-through") {
                        Some(true)
                    } else if parts.contains(&"none") {
                        Some(false)
                    } else {
                        None
                    },
                    ..Default::default()
                }
            }
            "intense" => Self {
                intense: Some(parse_boolean(value)),
                ..Default::default()
            },
            "dimmed" => Self {
                dimmed: Some(parse_boolean(value)),
                ..Default::default()
            },
            "reset" => Self {
                reset: Some(parse_boolean(value)),
                ..Default::default()
            },
            _ => Default::default(),
        }
    }
}

fn parse_boolean(value: &str) -> bool {
    !value.is_empty() && value != "false" && value != "none"
}

fn parse_color(value: &str) -> Option<Color> {
    fn parse_hex(hex: &str) -> Option<u8> {
        u8::from_str_radix(hex, 16).ok()
    }

    let value = value.trim();

    if value.is_empty()
        || value.eq_ignore_ascii_case("none")
        || value.eq_ignore_ascii_case("transparent")
    {
        return None;
    }

    if let Some(hex) = value.strip_prefix('#') {
        if hex.len() == 6 {
            let r = parse_hex(&hex[0..2])?;
            let g = parse_hex(&hex[2..4])?;
            let b = parse_hex(&hex[4..6])?;
            return Some(Color::Rgb(r, g, b));
        }
        if hex.len() == 3 {
            let r = parse_hex(&hex[0..1])?;
            let g = parse_hex(&hex[1..2])?;
            let b = parse_hex(&hex[2..3])?;
            return Some(Color::Rgb(r * 17, g * 17, b * 17));
        }
        return None;
    }

    let lower = value.to_ascii_lowercase();

    if let Some(ansi) = value.strip_prefix("ansi-") {
        return Some(Color::Ansi256(ansi.parse().ok()?));
    }

    Some(match lower.as_str() {
        "black" => Color::Black,
        "blue" => Color::Blue,
        "green" => Color::Green,
        "red" => Color::Red,
        "cyan" => Color::Cyan,
        "magenta" => Color::Magenta,
        "yellow" => Color::Yellow,
        "white" => Color::White,
        _ => {
            return None;
        }
    })
}

impl CssElement for Element {
    fn parent_element(&self) -> Option<Self> {
        Some(match self {
            Element::Value(Some(_)) => Element::Value(None),
            Element::Ast(Some(_)) => Element::Ast(None),
            Element::Markdown(Some(MarkdownElement::List {
                element: Some(List::Item),
                style,
            })) => Element::Markdown(Some(MarkdownElement::List {
                element: None,
                style: *style,
            })),
            Element::Markdown(Some(MarkdownElement::List {
                element: Some(List::Marker),
                style,
            })) => Element::Markdown(Some(MarkdownElement::List {
                element: Some(List::Item),
                style: *style,
            })),
            Element::Markdown(Some(_)) => Element::Markdown(None),
            Element::Prompt(Some(_)) => Element::Prompt(None),
            Element::Error(Some(_)) => Element::Error(None),
            _ => return None,
        })
    }

    fn prev_sibling_element(&self) -> Option<Self> {
        None
    }

    fn has_local_name(&self, name: &str) -> bool {
        match self {
            Element::Fluff => name == "dices-fluff",
            Element::Value(None) => name == "dices-value",
            Element::Value(Some(ValueElement::Null)) => name == "dices-value-null",
            Element::Value(Some(ValueElement::Bool { .. })) => name == "dices-value-bool",
            Element::Value(Some(ValueElement::Integer)) => name == "dices-value-integer",
            Element::Value(Some(ValueElement::String { .. })) => name == "dices-value-string",
            Element::Value(Some(ValueElement::Delimiter { .. })) => name == "dices-value-delimiter",
            Element::Value(Some(ValueElement::Punctuator)) => name == "dices-value-punctuator",
            Element::Value(Some(ValueElement::Injected)) => name == "dices-value-injected",
            Element::Ast(None) => name == "dices-ast",
            Element::Ast(Some(AstElement::Ident)) => name == "dices-ast-ident",
            Element::Markdown(None) => name == "dices-markdown",
            Element::Markdown(Some(MarkdownElement::Paragraph)) => name == "p",
            Element::Markdown(Some(MarkdownElement::Header { level })) => {
                matches!(
                    (name, level),
                    ("h1", 1) | ("h2", 2) | ("h3", 3) | ("h4", 4) | ("h5", 5) | ("h6", 6)
                )
            }
            Element::Markdown(Some(MarkdownElement::Code { inline: false })) => name == "pre",
            Element::Markdown(Some(MarkdownElement::Code { inline: true })) => name == "code",
            Element::Markdown(Some(MarkdownElement::Bold)) => name == "strong",
            Element::Markdown(Some(MarkdownElement::Italic)) => name == "em",
            Element::Markdown(Some(MarkdownElement::List {
                style: ListStyle::Unordered,
                element: None,
            })) => name == "ul",
            Element::Markdown(Some(MarkdownElement::List {
                style: ListStyle::Ordered,
                element: None,
            })) => name == "ol",
            Element::Markdown(Some(MarkdownElement::List {
                element: Some(List::Item),
                ..
            })) => name == "li",
            Element::Markdown(Some(MarkdownElement::List {
                element: Some(List::Marker),
                ..
            })) => name == "dices-markdown-list-marker",
            Element::Markdown(Some(MarkdownElement::Link { .. })) => name == "a",
            Element::Prompt(None) => name == "dices-prompt",
            Element::Prompt(Some(PromptElement::Indicator)) => name == "dices-prompt-indicator",
            Element::Prompt(Some(PromptElement::Multiline)) => name == "dices-prompt-multiline",
            Element::Prompt(Some(PromptElement::Right)) => name == "dices-prompt-right",
            Element::Error(None) => name == "dices-error",
            Element::Error(Some(ErrorElement::Message)) => name == "dices-error-message",
            Element::Error(Some(ErrorElement::Cause)) => name == "dices-error-cause",
        }
    }

    fn attribute_matches(&self, local_name: &str, operator: AttributeOperator<'_>) -> bool {
        fn bool_str(value: &bool) -> &'static str {
            match value {
                true => "true",
                false => "false",
            }
        }
        fn print_u8<'b, const N: usize>(buf: &'b mut [u8; N], value: &u8) -> &'b str {
            let mut dest = &mut buf[..];
            write!(&mut dest, "{value}").unwrap();
            let written = N - dest.len();
            str::from_utf8(&buf[..written]).unwrap()
        }

        let mut buf = [0; 3];

        let value = match (self, local_name) {
            (Element::Markdown(Some(MarkdownElement::Header { level })), "level") => {
                print_u8(&mut buf, level)
            }
            (Element::Markdown(Some(MarkdownElement::Code { inline })), "inline") => {
                bool_str(inline)
            }
            (Element::Value(Some(ValueElement::Bool { value })), "value") => bool_str(value),
            (Element::Value(Some(ValueElement::String { escape })), "escape") => bool_str(escape),
            (Element::Value(Some(ValueElement::Delimiter { kind, .. })), "kind") => match kind {
                DelimiterKind::List => "list",
                DelimiterKind::Map => "map",
            },
            (Element::Value(Some(ValueElement::Delimiter { nesting, .. })), "depth") => {
                print_u8(&mut buf, nesting)
            }
            (Element::Value(Some(ValueElement::Delimiter { nesting, .. })), s)
                if let Some(modulus) =
                    s.strip_prefix("depth-").and_then(|m| m.parse::<u8>().ok()) =>
            {
                print_u8(&mut buf, &(nesting % modulus))
            }

            (Element::Markdown(Some(MarkdownElement::Link { url })), "href") => url.as_str(),

            _ => return false,
        };

        operator.matches(value)
    }

    fn pseudo_class_matches(&self, _class: PseudoClass<'_>) -> bool {
        false
    }
}
