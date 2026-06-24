use figment::{Figment, providers::Serialized, value::Value};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};

use crate::{
    Element, AstElement, DelimiterKind, ErrorElement, MarkdownElement, PromptElement,
    ValueElement,
};

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
    ast: T,
    ast_ident: T,
    markdown: T,
    markdown_header: Box<[T]>,
    markdown_inline_code: T,
    markdown_bold: T,
    markdown_italic: T,
    markdown_list: T,
    markdown_list_item: T,
    markdown_list_marker_ordered: T,
    markdown_list_marker_unordered: T,
    prompt: T,
    prompt_indicator: T,
    prompt_multiline: T,
    prompt_right: T,
    error: T,
    error_message: T,
    error_source: T,
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
            ast: Self::extract(figment.clone(), &["ast"])?,
            ast_ident: Self::extract(figment.clone(), &["ast", "ident"])?,
            markdown: Self::extract(figment.clone(), &["markdown"])?,
            markdown_header: Self::extract_with_depth(figment.clone(), &["markdown", "header"])?,
            markdown_inline_code: Self::extract(figment.clone(), &["markdown", "inline_code"])?,
            markdown_bold: Self::extract(figment.clone(), &["markdown", "bold_text"])?,
            markdown_italic: Self::extract(figment.clone(), &["markdown", "italic_text"])?,
            markdown_list: Self::extract(figment.clone(), &["markdown", "list"])?,
            markdown_list_item: Self::extract(figment.clone(), &["markdown", "list", "item"])?,
            markdown_list_marker_ordered: Self::extract(
                figment.clone(),
                &["markdown", "list", "marker", "ordered"],
            )?,
            markdown_list_marker_unordered: Self::extract(
                figment.clone(),
                &["markdown", "list", "marker", "unordered"],
            )?,
            prompt: Self::extract(figment.clone(), &["prompt"])?,
            prompt_indicator: Self::extract(figment.clone(), &["prompt", "indicator"])?,
            prompt_multiline: Self::extract(figment.clone(), &["prompt", "multiline"])?,
            prompt_right: Self::extract(figment.clone(), &["prompt", "right"])?,
            error: Self::extract(figment.clone(), &["error"])?,
            error_message: Self::extract(figment.clone(), &["error", "message"])?,
            error_source: Self::extract(figment.clone(), &["error", "source"])?,
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
            ast: f(self.ast),
            ast_ident: f(self.ast_ident),
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
            markdown_list: f(self.markdown_list),
            markdown_list_item: f(self.markdown_list_item),
            markdown_list_marker_ordered: f(self.markdown_list_marker_ordered),
            markdown_list_marker_unordered: f(self.markdown_list_marker_unordered),
            prompt: f(self.prompt),
            prompt_indicator: f(self.prompt_indicator),
            prompt_multiline: f(self.prompt_multiline),
            prompt_right: f(self.prompt_right),
            error: f(self.error),
            error_message: f(self.error_message),
            error_source: f(self.error_source),
        }
    }

    pub fn style(&self, annotation: Element) -> &T {
        use Element::*;
        use AstElement::*;
        use DelimiterKind::*;
        use ErrorElement::*;
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
            Value(Some(Delimiter {
                kind: DelimiterKind::List,
                depth,
            })) => &self.value_delimiter_list[depth as usize % self.value_delimiter_list.len()],
            Value(Some(Delimiter { kind: Map, depth })) => {
                &self.value_delimiter_list[depth as usize % self.value_delimiter_list.len()]
            }
            Value(Some(Punctuator)) => &self.value_punctuator,
            Value(Some(Injected)) => &self.value_injected,
            Ast(None) => &self.ast,
            Ast(Some(Ident)) => &self.ast_ident,
            Markdown(None) => &self.markdown,
            Markdown(Some(Header { level })) => {
                &self.markdown_header[level as usize % self.markdown_header.len()]
            }
            Markdown(Some(InlineCode)) => &self.markdown_inline_code,
            Markdown(Some(Bold)) => &self.markdown_bold,
            Markdown(Some(Italic)) => &self.markdown_italic,
            Markdown(Some(MarkdownElement::List { element: None, .. })) => &self.markdown_list,
            Markdown(Some(MarkdownElement::List {
                element: Some(crate::List::Item),
                ..
            })) => &self.markdown_list_item,
            Markdown(Some(MarkdownElement::List {
                style: crate::ListStyle::Ordered,
                element: Some(crate::List::Marker),
            })) => &self.markdown_list_marker_ordered,
            Markdown(Some(MarkdownElement::List {
                style: crate::ListStyle::Unordered,
                element: Some(crate::List::Marker),
            })) => &self.markdown_list_marker_unordered,
            Prompt(None) => &self.prompt,
            Prompt(Some(Indicator)) => &self.prompt_indicator,
            Prompt(Some(Multiline)) => &self.prompt_multiline,
            Prompt(Some(Right)) => &self.prompt_right,
            Error(None) => &self.error,
            Error(Some(Message)) => &self.error_message,
            Error(Some(Source)) => &self.error_source,
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
