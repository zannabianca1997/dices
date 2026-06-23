//! Parsing of values

use std::{fmt, str::FromStr};

use dices_ast::identifier::Identifier;
use dices_values::{
    Value,
    bool::ValueBool,
    injected::ValueInjected,
    int::ValueInt,
    list::ValueList,
    map::ValueMap,
    null::ValueNull,
    string::{EscapeError, ValueString},
};

use itertools::Itertools;
use pest::{Parser, iterators::Pair};
use pest_derive::Parser;
use snafu::{ResultExt, Snafu};

#[derive(Parser)]
#[grammar = "value.pest"]
struct Grammar;

#[derive(Debug, Snafu)]
#[cfg_attr(test, derive(derive_more::IsVariant))]
pub enum ParseError {
    #[snafu(display("Pest parse error"))]
    Pest { source: pest::error::Error<Rule> },
    #[snafu(display("Failed to parse integer"))]
    IntParse { source: <ValueInt as FromStr>::Err },
    #[snafu(display("Failed to unescape string"))]
    StringUnescape { source: EscapeError },
    #[snafu(display("Invalid identifier: {text}"))]
    InvalidIdentifier { text: ValueString },
}

pub fn parse_value(input: &ValueString) -> Result<Value, ParseError> {
    let raw = input.as_str();
    let mut pairs = Grammar::parse(Rule::main, raw).context(PestSnafu)?;
    let value_pair = pairs.next().unwrap();
    build_value(value_pair, input)
}

fn build_value(value_pair: Pair<'_, Rule>, input: &ValueString) -> Result<Value, ParseError> {
    match value_pair.as_rule() {
        // `main` and `value` are thin wrappers around a single inner pair: descend.
        Rule::main | Rule::value => {
            let inner = value_pair.into_inner().next().unwrap();
            build_value(inner, input)
        }
        Rule::null => Ok(Value::Null(ValueNull)),
        Rule::bool => {
            let b = match value_pair.as_str() {
                "true" => ValueBool::TRUE,
                "false" => ValueBool::FALSE,
                _ => unreachable!(),
            };
            Ok(Value::Bool(b))
        }
        Rule::int => {
            let i = ValueInt::from_str(value_pair.as_str()).context(IntParseSnafu)?;
            Ok(Value::Int(i))
        }
        Rule::string => {
            let s = build_string(value_pair, input)?;
            Ok(Value::String(s))
        }
        Rule::list => {
            let items: Result<ValueList, _> = value_pair
                .into_inner()
                .map(|p| build_value(p, input))
                .collect();
            Ok(Value::List(items?))
        }
        Rule::map => {
            let mut pairs = value_pair.into_inner();
            let mut items = Vec::new();
            while let Some(key_pair) = pairs.next() {
                let value_pair = pairs.next().unwrap();
                let key = build_map_key(key_pair, input)?;
                let value = build_value(value_pair, input)?;
                items.push((key, value));
            }
            Ok(Value::Map(items.into_iter().collect::<ValueMap>()))
        }
        r => unreachable!("Unexpected rule {r:?}"),
    }
}

/// Unescape the contents of a `string` pair into a [`ValueString`].
fn build_string(pair: Pair<'_, Rule>, input: &ValueString) -> Result<ValueString, ParseError> {
    let span = pair.as_span();
    // Trim the surrounding quotes before unescaping.
    let range = (span.start() + 1)..(span.end() - 1);
    input
        .slice(range)
        .unwrap()
        .unescape()
        .context(StringUnescapeSnafu)
}

/// Build the key of a map entry, accepting either a quoted string or a bare identifier.
fn build_map_key(pair: Pair<'_, Rule>, input: &ValueString) -> Result<ValueString, ParseError> {
    match pair.as_rule() {
        Rule::string => build_string(pair, input),
        Rule::identifier => {
            // bare identifier key: use its raw text directly (no escapes possible).
            let span = pair.as_span();
            Ok(input.slice(span.start()..span.end()).unwrap())
        }
        r => unreachable!("Unexpected rule {r:?}"),
    }
}

#[derive(Debug, Snafu)]
pub enum PrintError {
    #[snafu(transparent)]
    Fmt { source: fmt::Error },
    #[snafu(display("Impossible to serialized {}", value.description()))]
    Injected { value: ValueInjected },
}

/// Prints values so they roundtrip with the parser
pub fn print(value: &Value, w: &mut impl fmt::Write) -> Result<(), PrintError> {
    match value {
        Value::List(value) => {
            write!(w, "<|")?;
            for (pos, el) in value.iter().with_position() {
                print(el, w)?;
                if !pos.is_last() {
                    write!(w, ", ")?;
                }
            }
            write!(w, "|>")?;
        }
        Value::Map(value) => {
            write!(w, "<|")?;
            for (pos, (k, v)) in value.iter().with_position() {
                if let Some(ident) = Identifier::new_ref(k) {
                    write!(w, "{ident}: ")?;
                } else {
                    write!(w, "{k}: ")?;
                }
                print(v, w)?;
                if !pos.is_last() {
                    write!(w, ", ")?;
                }
            }
            write!(w, "|>")?;
        }
        Value::Injected(value) => {
            return Err(PrintError::Injected {
                value: value.clone(),
            });
        }
        scalar => write!(w, "{scalar}")?,
    };
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use dices_values::{
        Value, bool::ValueBool, int::ValueInt, list::ValueList, map::ValueMap, null::ValueNull,
        string::ValueString,
    };

    use super::{ParseError, parse_value};

    fn parse(input: &'static str) -> Value {
        parse_value(&ValueString::new_static(input)).expect("parse_value should succeed")
    }

    fn parse_err(input: &'static str) -> ParseError {
        parse_value(&ValueString::new_static(input)).unwrap_err()
    }

    fn int(n: &str) -> Value {
        Value::Int(ValueInt::from_str(n).unwrap())
    }

    fn string(s: &'static str) -> Value {
        Value::String(ValueString::new_static(s))
    }

    #[test]
    fn null() {
        assert_eq!(parse("null"), Value::Null(ValueNull));
    }

    #[test]
    fn booleans() {
        assert_eq!(parse("true"), Value::Bool(ValueBool::TRUE));
        assert_eq!(parse("false"), Value::Bool(ValueBool::FALSE));
    }

    #[test]
    fn integers() {
        assert_eq!(parse("42"), int("42"));
        assert_eq!(parse("-7"), int("-7"));
        assert_eq!(parse("0"), int("0"));
    }

    #[test]
    fn strings() {
        assert_eq!(parse(r#""hello""#), string("hello"));
        assert_eq!(parse(r#""""#), string(""));
        assert_eq!(parse(r#""a\nb\tc\"d\\e""#), string("a\nb\tc\"d\\e"));
    }

    #[test]
    fn lists() {
        assert_eq!(parse("[]"), Value::List(ValueList::from_iter([])));
        assert_eq!(
            parse("[1, 2, 3,]"),
            Value::List(ValueList::from_iter([int("1"), int("2"), int("3")]))
        );
        assert_eq!(
            parse(r#"[1, "x", true]"#),
            Value::List(ValueList::from_iter([
                int("1"),
                string("x"),
                Value::Bool(ValueBool::TRUE),
            ]))
        );
    }

    #[test]
    fn maps() {
        assert_eq!(parse("<| |>"), Value::Map(ValueMap::from_iter([])));
        assert_eq!(
            parse(r#"<| "a": 1, b: 2, |>"#),
            Value::Map(ValueMap::from_iter([
                (ValueString::new_static("a"), int("1")),
                (ValueString::new_static("b"), int("2")),
            ]))
        );
    }

    #[test]
    fn nested() {
        assert_eq!(
            parse(r#"<| items: [1, <| k: "v" |>] |>"#),
            Value::Map(ValueMap::from_iter([(
                ValueString::new_static("items"),
                Value::List(ValueList::from_iter([
                    int("1"),
                    Value::Map(ValueMap::from_iter([(
                        ValueString::new_static("k"),
                        string("v"),
                    )])),
                ])),
            )]))
        );
    }

    #[test]
    fn errors() {
        assert!(parse_err("foo").is_pest());
        assert!(parse_err(r#""unterminated"#).is_pest());
        assert!(parse_err("").is_pest());
    }
}
