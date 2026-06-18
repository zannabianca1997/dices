use figment::{Figment, providers::Serialized, value::Value};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{Annotation, DelimiterKind, ValueElement};

/// Themes for the 
#[derive(Debug, Deserialize, Serialize, Default)]
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
}

impl<T> Theme<T> {
    pub fn new(figment: Figment) -> figment::Result<Self>
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

    pub fn style(&self, annotation: Annotation) -> &T {
        use Annotation::*;
        use DelimiterKind::*;
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
        }
    }
}
