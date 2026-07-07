use std::{borrow::Cow, fmt::Display};

pub use optional_struct::Applicable as MergeStyle;
use optional_struct::optional_struct;
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

#[derive(Debug, Clone, Copy)]
#[optional_struct(Style)]
pub struct FullStyle {
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

impl Copy for Style {}

impl Default for FullStyle {
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
                    Style::from_declaration(decl.name, decl.value).apply_to_opt(&mut style);
                }
            }
        }

        style
    }

    pub fn to_css(&self) -> impl Display {
        &self.stylesheet
    }
}

impl Default for Theme<'_> {
    fn default() -> Self {
        Self {
            stylesheet: StyleSheet::default(),
        }
    }
}

impl Style {
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

impl Element {
    /// The HTML/custom-element tag name representing this element.
    ///
    /// Used by the theming style sheet
    pub fn local_name(&self) -> &'static str {
        match self {
            Element::Fluff => "dices-fluff",
            Element::Value(None) => "dices-value",
            Element::Value(Some(ValueElement::Null)) => "dices-value-null",
            Element::Value(Some(ValueElement::Bool { .. })) => "dices-value-bool",
            Element::Value(Some(ValueElement::Integer)) => "dices-value-integer",
            Element::Value(Some(ValueElement::String { .. })) => "dices-value-string",
            Element::Value(Some(ValueElement::Delimiter { .. })) => "dices-value-delimiter",
            Element::Value(Some(ValueElement::Punctuator)) => "dices-value-punctuator",
            Element::Value(Some(ValueElement::Injected)) => "dices-value-injected",
            Element::Ast(None) => "dices-ast",
            Element::Ast(Some(AstElement::Ident)) => "dices-ast-ident",
            Element::Markdown(None) => "dices-markdown",
            Element::Markdown(Some(MarkdownElement::Paragraph)) => "p",
            Element::Markdown(Some(MarkdownElement::Header { level })) => match level {
                1 => "h1",
                2 => "h2",
                3 => "h3",
                4 => "h4",
                5 => "h5",
                _ => "h6",
            },
            Element::Markdown(Some(MarkdownElement::Code { inline: false })) => "pre",
            Element::Markdown(Some(MarkdownElement::Code { inline: true })) => "code",
            Element::Markdown(Some(MarkdownElement::Bold)) => "strong",
            Element::Markdown(Some(MarkdownElement::Italic)) => "em",
            Element::Markdown(Some(MarkdownElement::List {
                style: ListStyle::Unordered,
                element: None,
            })) => "ul",
            Element::Markdown(Some(MarkdownElement::List {
                style: ListStyle::Ordered,
                element: None,
            })) => "ol",
            Element::Markdown(Some(MarkdownElement::List {
                element: Some(List::Item),
                ..
            })) => "li",
            Element::Markdown(Some(MarkdownElement::List {
                element: Some(List::Marker),
                ..
            })) => "dices-markdown-list-marker",
            Element::Markdown(Some(MarkdownElement::Link { .. })) => "a",
            Element::Prompt(None) => "dices-prompt",
            Element::Prompt(Some(PromptElement::Indicator)) => "dices-prompt-indicator",
            Element::Prompt(Some(PromptElement::Multiline)) => "dices-prompt-multiline",
            Element::Prompt(Some(PromptElement::Right)) => "dices-prompt-right",
            Element::Error(None) => "dices-error",
            Element::Error(Some(ErrorElement::Message)) => "dices-error-message",
            Element::Error(Some(ErrorElement::Cause)) => "dices-error-cause",
        }
    }

    /// The HTML/custom-element attributes carrying this element's data.
    ///
    /// Source of truth for both the CSS attribute matcher below and HTML
    /// renderers. Does not include the computed `depth-N` selector, which has
    /// no single fixed value (see [`CssElement::attribute_matches`]).
    pub fn attributes(&self) -> Vec<(&'static str, Cow<'_, str>)> {
        fn bool_str(value: &bool) -> &'static str {
            match value {
                true => "true",
                false => "false",
            }
        }

        match self {
            Element::Markdown(Some(MarkdownElement::Header { level })) => {
                vec![("data-level", Cow::Owned(level.to_string()))]
            }
            Element::Markdown(Some(MarkdownElement::Code { inline })) => {
                vec![("data-inline", Cow::Borrowed(bool_str(inline)))]
            }
            Element::Value(Some(ValueElement::Bool { value })) => {
                vec![("data-value", Cow::Borrowed(bool_str(value)))]
            }
            Element::Value(Some(ValueElement::String { escape })) => {
                vec![("data-escape", Cow::Borrowed(bool_str(escape)))]
            }
            Element::Value(Some(ValueElement::Delimiter { kind, nesting })) => vec![
                (
                    "data-kind",
                    Cow::Borrowed(match kind {
                        DelimiterKind::List => "list",
                        DelimiterKind::Map => "map",
                    }),
                ),
                ("data-depth", Cow::Owned(nesting.to_string())),
            ],
            Element::Markdown(Some(MarkdownElement::Link { url })) => {
                vec![("href", Cow::Borrowed(url.as_str()))]
            }
            _ => Vec::new(),
        }
    }
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
        self.local_name() == name
    }

    fn attribute_matches(&self, local_name: &str, operator: AttributeOperator<'_>) -> bool {
        // special `depth-N` selector creates dummy `depth-N` attributes on the
        // fly to match the styled css
        if let Element::Value(Some(ValueElement::Delimiter { nesting, .. })) = self
            && let Some(modulus) = local_name
                .strip_prefix("data-depth-")
                .and_then(|m| m.parse::<u8>().ok())
        {
            return operator.matches(&(nesting % modulus).to_string());
        }

        self.attributes()
            .into_iter()
            .find(|(name, _)| *name == local_name)
            .is_some_and(|(_, value)| operator.matches(&value))
    }

    fn pseudo_class_matches(&self, _class: PseudoClass<'_>) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[test]
    fn local_name_matches_expected_tags() {
        assert_eq!(Element::Fluff.local_name(), "dices-fluff");
        assert_eq!(
            Element::Markdown(Some(MarkdownElement::Header { level: 3 })).local_name(),
            "h3"
        );
        assert_eq!(
            Element::Markdown(Some(MarkdownElement::Link {
                url: Url::parse("https://example.com/").unwrap()
            }))
            .local_name(),
            "a"
        );
    }

    #[test]
    fn css_matches_depth_and_href_selectors() {
        let theme = Theme::parse(
            "dices-value-delimiter[data-depth-3=\"1\"] { color: red; }\n\
             a[href=\"https://example.com/\"] { color: blue; }",
        );

        let delimiter = Element::Value(Some(ValueElement::Delimiter {
            kind: DelimiterKind::List,
            nesting: 4, // 4 % 3 == 1
        }));
        assert_eq!(theme.style(&delimiter).fg_color, Some(Color::Red));

        let link = Element::Markdown(Some(MarkdownElement::Link {
            url: Url::parse("https://example.com/").unwrap(),
        }));
        assert_eq!(theme.style(&link).fg_color, Some(Color::Blue));
    }
}
