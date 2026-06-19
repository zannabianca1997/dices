use figment::{Figment, providers::Serialized, value::Value};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};

use crate::{Annotation, DelimiterKind, MarkdownElement, PromptElement, ValueElement};

/// Themes for the
#[derive(Debug, Default, PartialEq)]
pub struct Theme<T> {
    fluff: T,
    value: T,
    value_null: T,
    value_integer: T,
    value_bool_true: T,
    value_bool_false: T,
    value_string: T,
    value_string_escape: T,
    value_delimiter_list: Box<[T]>,
    value_delimiter_map: Box<[T]>,
    value_punctuator: T,
    value_injected: T,
    markdown: T,
    markdown_header: Box<[T]>,
    markdown_inline_code: T,
    markdown_bold: T,
    markdown_italic: T,
    prompt: T,
    prompt_indicator: T,
    prompt_multiline: T,
    prompt_right: T,
}

impl<T> Theme<T> {
    fn from_figment(figment: Figment) -> figment::Result<Self>
    where
        T: DeserializeOwned,
    {
        Ok(Self {
            fluff: Self::extract(figment.clone(), &["fluff"])?,
            value: Self::extract(figment.clone(), &["value"])?,
            value_null: Self::extract(figment.clone(), &["value", "null"])?,
            value_integer: Self::extract(figment.clone(), &["value", "integer"])?,
            value_bool_true: Self::extract(figment.clone(), &["value", "bool", "true"])?,
            value_bool_false: Self::extract(figment.clone(), &["value", "bool", "false"])?,
            value_string: Self::extract(figment.clone(), &["value", "string"])?,
            value_string_escape: Self::extract(figment.clone(), &["value", "string", "escape"])?,
            value_delimiter_list: Self::extract_with_depth(
                figment.clone(),
                &["value", "delimiter", "list"],
            )?,
            value_delimiter_map: Self::extract_with_depth(
                figment.clone(),
                &["value", "delimiter", "map"],
            )?,
            value_punctuator: Self::extract(figment.clone(), &["value", "punctuator"])?,
            value_injected: Self::extract(figment.clone(), &["value", "injected"])?,
            markdown: Self::extract(figment.clone(), &["markdown"])?,
            markdown_header: Self::extract_with_depth(figment.clone(), &["markdown", "header"])?,
            markdown_inline_code: Self::extract(figment.clone(), &["markdown", "inline_code"])?,
            markdown_bold: Self::extract(figment.clone(), &["markdown", "bold_text"])?,
            markdown_italic: Self::extract(figment.clone(), &["markdown", "italic_text"])?,
            prompt: Self::extract(figment.clone(), &["prompt"])?,
            prompt_indicator: Self::extract(figment.clone(), &["prompt", "indicator"])?,
            prompt_multiline: Self::extract(figment.clone(), &["prompt", "multiline"])?,
            prompt_right: Self::extract(figment.clone(), &["prompt", "right"])?,
        })
    }

    fn extract(mut figment: Figment, path: &'static [&'static str]) -> figment::Result<T>
    where
        T: DeserializeOwned,
    {
        let mut total = figment.clone();
        for key in path {
            figment = figment.focus(key);
            total = total.merge(figment.clone());
        }
        total.extract()
    }

    fn extract_with_depth(
        mut figment: Figment,
        path: &'static [&'static str],
    ) -> figment::Result<Box<[T]>>
    where
        T: DeserializeOwned,
    {
        let mut total = figment.clone();
        for key in path {
            figment = figment.focus(key);
            total = total.merge(figment.clone());
        }
        let depth: Vec<Value> = total.extract_inner("depth")?;

        depth
            .into_iter()
            .map(|v| {
                Figment::from(Serialized::defaults(v))
                    .join(total.clone())
                    .extract()
            })
            .collect()
    }

    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> Theme<U> {
        Theme {
            fluff: f(self.fluff),
            value: f(self.value),
            value_null: f(self.value_null),
            value_integer: f(self.value_integer),
            value_bool_true: f(self.value_bool_true),
            value_bool_false: f(self.value_bool_false),
            value_string: f(self.value_string),
            value_string_escape: f(self.value_string_escape),
            value_delimiter_list: self
                .value_delimiter_list
                .into_vec()
                .into_iter()
                .map(&mut f)
                .collect(),
            value_delimiter_map: self
                .value_delimiter_map
                .into_vec()
                .into_iter()
                .map(&mut f)
                .collect(),
            value_punctuator: f(self.value_punctuator),
            value_injected: f(self.value_injected),
            markdown: f(self.markdown),
            markdown_header: self
                .markdown_header
                .into_vec()
                .into_iter()
                .map(&mut f)
                .collect(),
            markdown_inline_code: f(self.markdown_inline_code),
            markdown_bold: f(self.markdown_bold),
            markdown_italic: f(self.markdown_italic),
            prompt: f(self.prompt),
            prompt_indicator: f(self.prompt_indicator),
            prompt_multiline: f(self.prompt_multiline),
            prompt_right: f(self.prompt_right),
        }
    }

    #[allow(unreachable_patterns)]
    pub fn style(&self, annotation: Annotation) -> &T {
        use Annotation::*;
        use DelimiterKind::*;
        use MarkdownElement::*;
        use PromptElement::*;
        use ValueElement::*;

        match annotation {
            Fluff => &self.fluff,
            Value(None) => &self.value,
            Value(Some(Null)) => &self.value_null,
            Value(Some(Integer)) => &self.value_integer,
            Value(Some(Bool { value: true })) => &self.value_bool_true,
            Value(Some(Bool { value: false })) => &self.value_bool_false,
            Value(Some(String { escape: false })) => &self.value_string,
            Value(Some(String { escape: true })) => &self.value_string_escape,
            Value(Some(Delimiter { kind: List, depth })) => {
                &self.value_delimiter_list[depth as usize % self.value_delimiter_list.len()]
            }
            Value(Some(Delimiter { kind: Map, depth })) => {
                &self.value_delimiter_list[depth as usize % self.value_delimiter_list.len()]
            }
            Value(Some(Punctuator)) => &self.value_punctuator,
            Value(Some(Injected)) => &self.value_injected,
            Markdown(None) => &self.markdown,
            Markdown(Some(Header { level })) => {
                &self.markdown_header[level as usize % self.markdown_header.len()]
            }
            Markdown(Some(InlineCode)) => &self.markdown_inline_code,
            Markdown(Some(Bold)) => &self.markdown_bold,
            Markdown(Some(Italic)) => &self.markdown_italic,
            Markdown(Some(_)) => &self.markdown,
            Prompt(None) => &self.prompt,
            Prompt(Some(Indicator)) => &self.prompt_indicator,
            Prompt(Some(Multiline)) => &self.prompt_multiline,
            Prompt(Some(Right)) => &self.prompt_right,
        }
    }
}

impl<'de, T> Deserialize<'de> for Theme<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // `Value::deserialize` also names an inherent `&self` method, so the
        // bare path would be ambiguous: spell out the trait method.
        let value = <Value as Deserialize>::deserialize(deserializer)?;
        Self::from_figment(Figment::from(Serialized::defaults(value)))
            .map_err(serde::de::Error::custom)
    }
}
