//! Matcher syntax for values
//!
//! Used in for the examples in the manual, where after each line a matcher is
//! specified. They are parsed and matched against the actual result.

use std::{collections::BTreeMap, rc::Rc, str::FromStr};

use lazy_regex::regex::Regex;
use pest::Parser;
use pest_derive::Parser;
use snafu::{ResultExt, Snafu};

use dices_values::{
    Value,
    bool::ValueBool,
    cast::push_down_if_injected,
    int::ValueInt,
    map::ValueMap,
    null::ValueNull,
    string::{EscapeError, ValueString},
};

#[derive(Parser)]
#[grammar = "matcher.pest"]
struct Grammar;

#[derive(Debug, Snafu)]
pub enum ParseMatcherError {
    #[snafu(display("Pest parse error"))]
    Pest { source: pest::error::Error<Rule> },
    #[snafu(display("Failed to parse integer"))]
    IntParse { source: <ValueInt as FromStr>::Err },
    #[snafu(display("Failed to unescape string"))]
    StringUnescape { source: EscapeError },
    #[snafu(display("Failed to compile regex"))]
    InvalidRegex { source: lazy_regex::regex::Error },
}

pub fn parse_matcher(input: &ValueString) -> Result<Matcher, ParseMatcherError> {
    let raw = input.as_str();
    let mut pairs = Grammar::parse(Rule::main, raw).context(PestSnafu)?;
    let matcher_pair = pairs.next().unwrap();
    build_matcher(matcher_pair, input)
}

#[derive(Clone)]
pub struct Matcher(Rc<dyn Fn(&Value) -> bool>);

impl Matcher {
    pub fn new<F>(fun: F) -> Self
    where
        F: Fn(&Value) -> bool + 'static,
    {
        Self(Rc::new(fun))
    }

    pub fn matches(&self, value: &Value) -> bool {
        (self.0)(value)
    }

    pub fn exactly(value: Value) -> Self {
        Self::new(move |v| v == &value)
    }

    pub fn any() -> Self {
        Self::new(|_| true)
    }

    pub fn none() -> Self {
        Self::new(|_| false)
    }

    pub fn and(self, other: Self) -> Self {
        Self::new(move |v| self.matches(v) && other.matches(v))
    }

    pub fn or(self, other: Self) -> Self {
        Self::new(move |v| self.matches(v) || other.matches(v))
    }

    #[allow(clippy::should_implement_trait)]
    pub fn not(self) -> Self {
        Self::new(move |v| !self.matches(v))
    }

    pub fn int_range(
        lower: Option<ValueInt>,
        upper: Option<ValueInt>,
        upper_inclusive: bool,
    ) -> Self {
        Self::new(move |v| {
            let resolved = push_down_if_injected(v.clone()).unwrap();
            let Value::Int(n) = &resolved else {
                return false;
            };
            if let Some(ref lo) = lower
                && n < lo
            {
                return false;
            }
            if let Some(ref hi) = upper {
                if upper_inclusive {
                    if n > hi {
                        return false;
                    }
                } else if n >= hi {
                    return false;
                }
            }
            true
        })
    }

    pub fn regex(re: Regex) -> Self {
        Self::new(move |v| {
            let resolved = push_down_if_injected(v.clone()).unwrap();
            let Value::String(s) = &resolved else {
                return false;
            };
            re.is_match(s.as_str())
        })
    }

    pub fn injectable_any() -> Self {
        Self::new(|v| matches!(v, Value::Injected(_)))
    }

    pub fn injectable_desc(desc: ValueString) -> Self {
        Self::new(move |v| match v {
            Value::Injected(inj) => inj.description().to_string() == desc.as_str(),
            _ => false,
        })
    }

    pub fn list_exact(items: Vec<Matcher>) -> Self {
        Self::new(move |v| {
            let resolved = push_down_if_injected(v.clone()).unwrap();
            let Value::List(list) = &resolved else {
                return false;
            };
            if list.len() != items.len() {
                return false;
            }
            for (item, matcher) in list.iter().zip(&items) {
                if !matcher.matches(item) {
                    return false;
                }
            }
            true
        })
    }

    pub fn list_slice(prefix: Vec<Matcher>, middle: Option<Matcher>, suffix: Vec<Matcher>) -> Self {
        Self::new(move |v| {
            let resolved = push_down_if_injected(v.clone()).unwrap();
            let Value::List(list) = &resolved else {
                return false;
            };
            let min_len = prefix.len() + suffix.len();
            if list.len() < min_len {
                return false;
            }
            for (i, m) in prefix.iter().enumerate() {
                if !m.matches(&list[i]) {
                    return false;
                }
            }
            let middle_len = list.len() - min_len;
            let middle_start = prefix.len();
            let middle_end = middle_start + middle_len;
            if let Some(ref m) = middle {
                let slice = list
                    .slice(middle_start..middle_end)
                    .expect("slice bounds should be valid");
                if !m.matches(&Value::List(slice)) {
                    return false;
                }
            }
            for (j, m) in suffix.iter().enumerate() {
                let idx = middle_end + j;
                if !m.matches(&list[idx]) {
                    return false;
                }
            }
            true
        })
    }

    pub fn map_exact(entries: Vec<(ValueString, Matcher)>) -> Self {
        Self::new(move |v| {
            let resolved = push_down_if_injected(v.clone()).unwrap();
            let Value::Map(map) = &resolved else {
                return false;
            };
            if map.len() != entries.len() {
                return false;
            }
            for (key, matcher) in &entries {
                match map.get(key) {
                    Some(val) if matcher.matches(val) => {}
                    _ => return false,
                }
            }
            true
        })
    }

    pub fn map_rest(entries: Vec<(ValueString, Matcher)>, rest: Option<Matcher>) -> Self {
        Self::new(move |v| {
            let resolved = push_down_if_injected(v.clone()).unwrap();
            let Value::Map(map) = &resolved else {
                return false;
            };
            for (key, matcher) in &entries {
                match map.get(key) {
                    Some(val) if matcher.matches(val) => {}
                    _ => return false,
                }
            }
            if let Some(ref rest_matcher) = rest {
                let extra: BTreeMap<_, _> = map
                    .iter()
                    .filter(|(k, _)| !entries.iter().any(|(ek, _)| ek == *k))
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                if !rest_matcher.matches(&Value::Map(ValueMap::new(extra))) {
                    return false;
                }
            }
            true
        })
    }
}

fn build_matcher(
    pair: pest::iterators::Pair<Rule>,
    input: &ValueString,
) -> Result<Matcher, ParseMatcherError> {
    match pair.as_rule() {
        Rule::main | Rule::matcher | Rule::primary => {
            let inner = pair.into_inner().next().unwrap();
            build_matcher(inner, input)
        }
        Rule::paren => {
            let inner = pair.into_inner().next().unwrap();
            build_matcher(inner, input)
        }
        Rule::or_matcher => {
            let mut pairs = pair.into_inner();
            let first = build_matcher(pairs.next().unwrap(), input)?;
            let matchers: Result<Vec<_>, _> = pairs.map(|p| build_matcher(p, input)).collect();
            Ok(matchers?.into_iter().fold(first, Matcher::or))
        }
        Rule::and_matcher => {
            let mut pairs = pair.into_inner();
            let first = build_matcher(pairs.next().unwrap(), input)?;
            let matchers: Result<Vec<_>, _> = pairs.map(|p| build_matcher(p, input)).collect();
            Ok(matchers?.into_iter().fold(first, Matcher::and))
        }
        Rule::not_matcher => {
            let mut pairs = pair.into_inner();
            let inner = pairs.next().unwrap();
            let is_not = inner.as_rule() == Rule::not_matcher;
            let m = build_matcher(inner, input)?;
            if is_not { Ok(m.not()) } else { Ok(m) }
        }
        Rule::any => Ok(Matcher::any()),
        Rule::null => Ok(build_exact_null()),
        Rule::bool => {
            let b = match pair.as_str() {
                "true" => ValueBool::TRUE,
                "false" => ValueBool::FALSE,
                _ => unreachable!(),
            };
            Ok(build_exact_matcher(Value::Bool(b)))
        }
        Rule::int => {
            let i = ValueInt::from_str(pair.as_str()).context(IntParseSnafu)?;
            Ok(build_exact_matcher(Value::Int(i)))
        }
        Rule::string => {
            let s = build_string(pair, input)?;
            Ok(build_exact_matcher(Value::String(s)))
        }
        Rule::range => {
            let mut pairs = pair.into_inner();
            let first = pairs.next().unwrap();
            let lower = if first.as_rule() == Rule::int {
                let lo = ValueInt::from_str(first.as_str()).context(IntParseSnafu)?;
                pairs.next(); // skip range_sep
                Some(lo)
            } else {
                // first is range_sep, no lower bound
                None
            };
            let upper_info = pairs.next();
            let (upper, upper_inclusive) = match upper_info {
                Some(p) if p.as_rule() == Rule::range_upper_inclusive => {
                    let s = p.as_str();
                    let val = ValueInt::from_str(&s[1..]).context(IntParseSnafu)?;
                    (Some(val), true)
                }
                Some(p) => {
                    let val = ValueInt::from_str(p.as_str()).context(IntParseSnafu)?;
                    (Some(val), false)
                }
                None => (None, false),
            };
            Ok(Matcher::int_range(lower, upper, upper_inclusive))
        }
        Rule::regex => {
            let string_pair = pair.into_inner().next().unwrap();
            let pattern = build_raw_string_inner(string_pair, input);
            let re = Regex::new(&pattern).context(InvalidRegexSnafu)?;
            Ok(Matcher::regex(re))
        }
        Rule::injectable => {
            let inner = pair.as_str();
            let content = &inner[1..inner.len() - 1]; // strip < >
            if content == "..." {
                Ok(Matcher::injectable_any())
            } else {
                Ok(Matcher::injectable_desc(ValueString::from(
                    content.to_owned(),
                )))
            }
        }
        Rule::list => {
            let mut pairs = pair.into_inner();
            let Some(body) = pairs.next() else {
                return Ok(Matcher::list_exact(vec![]));
            };
            let mut body_pairs = body.into_inner();
            let Some(first) = body_pairs.next() else {
                return Ok(Matcher::list_exact(vec![]));
            };
            match first.as_rule() {
                Rule::elements => {
                    let prefix: Vec<Matcher> = first
                        .into_inner()
                        .map(|p| build_matcher(p, input))
                        .collect::<Result<_, _>>()?;
                    match body_pairs.next() {
                        Some(slice_pair) => {
                            let slice_matcher = build_slice_matcher(slice_pair, input)?;
                            let suffix: Vec<Matcher> = match body_pairs.next() {
                                Some(p) => p
                                    .into_inner()
                                    .map(|m| build_matcher(m, input))
                                    .collect::<Result<_, _>>()?,
                                None => vec![],
                            };
                            Ok(Matcher::list_slice(prefix, slice_matcher, suffix))
                        }
                        None => Ok(Matcher::list_exact(prefix)),
                    }
                }
                Rule::slice => {
                    let slice_matcher = build_slice_matcher(first, input)?;
                    let suffix: Vec<Matcher> = match body_pairs.next() {
                        Some(p) => p
                            .into_inner()
                            .map(|m| build_matcher(m, input))
                            .collect::<Result<_, _>>()?,
                        None => vec![],
                    };
                    Ok(Matcher::list_slice(vec![], slice_matcher, suffix))
                }
                r => unreachable!("Unexpected rule {r:?}"),
            }
        }
        Rule::map => {
            let mut pairs = pair.into_inner();
            let Some(body) = pairs.next() else {
                return Ok(Matcher::map_exact(vec![]));
            };
            let mut body_pairs = body.into_inner();
            let Some(first) = body_pairs.next() else {
                return Ok(Matcher::map_exact(vec![]));
            };
            match first.as_rule() {
                Rule::entries => {
                    let entries = build_map_entries(first, input)?;
                    match body_pairs.next() {
                        Some(slice_pair) => {
                            let rest = build_slice_matcher(slice_pair, input)?;
                            Ok(Matcher::map_rest(entries, rest))
                        }
                        None => Ok(Matcher::map_exact(entries)),
                    }
                }
                Rule::slice => {
                    let rest = build_slice_matcher(first, input)?;
                    Ok(Matcher::map_rest(vec![], rest))
                }
                r => unreachable!("Unexpected rule {r:?}"),
            }
        }
        r => unreachable!("Unexpected rule {r:?}"),
    }
}

fn build_slice_matcher(
    pair: pest::iterators::Pair<Rule>,
    input: &ValueString,
) -> Result<Option<Matcher>, ParseMatcherError> {
    let mut inner = pair.into_inner();
    inner.next().map(|p| build_matcher(p, input)).transpose()
}

fn build_map_entries(
    pair: pest::iterators::Pair<Rule>,
    input: &ValueString,
) -> Result<Vec<(ValueString, Matcher)>, ParseMatcherError> {
    pair.into_inner()
        .map(|entry| {
            let mut inner = entry.into_inner();
            let key_pair = inner.next().unwrap();
            let matcher_pair = inner.next().unwrap();
            let key = build_map_key(key_pair, input)?;
            let m = build_matcher(matcher_pair, input)?;
            Ok((key, m))
        })
        .collect()
}

fn build_map_key(
    pair: pest::iterators::Pair<Rule>,
    input: &ValueString,
) -> Result<ValueString, ParseMatcherError> {
    match pair.as_rule() {
        Rule::string => build_string(pair, input),
        Rule::identifier => {
            let span = pair.as_span();
            Ok(input.slice(span.start()..span.end()).unwrap())
        }
        r => unreachable!("Unexpected rule {r:?}"),
    }
}

fn build_string(
    pair: pest::iterators::Pair<Rule>,
    input: &ValueString,
) -> Result<ValueString, ParseMatcherError> {
    let span = pair.as_span();
    let range = (span.start() + 1)..(span.end() - 1);
    input
        .slice(range)
        .unwrap()
        .unescape()
        .context(StringUnescapeSnafu)
}

fn build_raw_string_inner(pair: pest::iterators::Pair<Rule>, input: &ValueString) -> ValueString {
    let span = pair.as_span();
    let range = (span.start() + 1)..(span.end() - 1);
    input.slice(range).unwrap()
}

fn build_exact_matcher(value: Value) -> Matcher {
    Matcher::new(move |v| {
        let resolved = push_down_if_injected(v.clone()).unwrap();
        resolved == value
    })
}

fn build_exact_null() -> Matcher {
    build_exact_matcher(Value::Null(ValueNull))
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, str::FromStr};

    use dices_values::{
        Value, bool::ValueBool, int::ValueInt, list::ValueList, map::ValueMap, null::ValueNull,
        string::ValueString,
    };

    use super::parse_matcher;

    fn int(s: &str) -> ValueInt {
        ValueInt::from_str(s).unwrap()
    }

    fn parse(input: &'static str) -> super::Matcher {
        parse_matcher(&ValueString::new_static(input)).expect("parse_matcher should succeed")
    }

    fn m(value: &'static str) -> super::Matcher {
        parse(value)
    }

    #[test]
    fn superset_null() {
        let mat = m("null");
        assert!(mat.matches(&Value::Null(ValueNull)));
        assert!(!mat.matches(&Value::Int(int("0"))));
    }

    #[test]
    fn superset_bool() {
        let mat = m("true");
        assert!(mat.matches(&Value::Bool(ValueBool::TRUE)));
        assert!(!mat.matches(&Value::Bool(ValueBool::FALSE)));

        let mat = m("false");
        assert!(mat.matches(&Value::Bool(ValueBool::FALSE)));
        assert!(!mat.matches(&Value::Bool(ValueBool::TRUE)));
    }

    #[test]
    fn superset_int() {
        let mat = m("42");
        assert!(mat.matches(&Value::Int(int("42"))));
        assert!(!mat.matches(&Value::Int(int("43"))));
    }

    #[test]
    fn superset_string() {
        let mat = m(r#""hello""#);
        assert!(mat.matches(&Value::String(ValueString::new_static("hello"))));
        assert!(!mat.matches(&Value::String(ValueString::new_static("world"))));
    }

    #[test]
    fn superset_list_empty() {
        let mat = m("[]");
        assert!(mat.matches(&Value::List(ValueList::empty())));
        assert!(
            !mat.matches(&Value::List(ValueList::from_iter(vec![Value::Int(int(
                "1"
            ))])))
        );
    }

    #[test]
    fn superset_list_exact() {
        let mat = m("[1, 2, 3]");
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("2")),
            Value::Int(int("3")),
        ]))));
        assert!(!mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("2")),
        ]))));
    }

    #[test]
    fn superset_map_empty() {
        let mat = m("<| |>");
        assert!(mat.matches(&Value::Map(ValueMap::new(BTreeMap::new()))));
    }

    #[test]
    fn superset_map_exact() {
        let mat = m(r#"<| a: 1, b: "hello" |>"#);
        let map = ValueMap::new(BTreeMap::from([
            (ValueString::new_static("a"), Value::Int(int("1"))),
            (
                ValueString::new_static("b"),
                Value::String(ValueString::new_static("hello")),
            ),
        ]));
        assert!(mat.matches(&Value::Map(map)));
    }

    #[test]
    fn any_matches_everything() {
        let mat = m("_");
        assert!(mat.matches(&Value::Null(ValueNull)));
        assert!(mat.matches(&Value::Int(int("0"))));
        assert!(mat.matches(&Value::String(ValueString::new_static("hi"))));
        assert!(mat.matches(&Value::List(ValueList::empty())));
        assert!(mat.matches(&Value::Map(ValueMap::new(BTreeMap::new()))));
    }

    #[test]
    fn and_operator() {
        let mat = m("_ & _");
        assert!(mat.matches(&Value::Int(int("1"))));
    }

    #[test]
    fn and_operator_rejects() {
        let mat = m("42 & 43");
        assert!(!mat.matches(&Value::Int(int("42"))));
    }

    #[test]
    fn or_operator() {
        let mat = m("42 | 43");
        assert!(mat.matches(&Value::Int(int("42"))));
        assert!(mat.matches(&Value::Int(int("43"))));
        assert!(!mat.matches(&Value::Int(int("44"))));
    }

    #[test]
    fn not_operator() {
        let mat = m("!42");
        assert!(!mat.matches(&Value::Int(int("42"))));
        assert!(mat.matches(&Value::Int(int("43"))));
    }

    #[test]
    fn double_not() {
        let mat = m("!!42");
        assert!(mat.matches(&Value::Int(int("42"))));
    }

    #[test]
    fn paren_grouping() {
        let mat = m("(42 | 43) & !43");
        assert!(mat.matches(&Value::Int(int("42"))));
        assert!(!mat.matches(&Value::Int(int("43"))));
    }

    #[test]
    fn range_all_ints() {
        let mat = m("..");
        assert!(mat.matches(&Value::Int(int("0"))));
        assert!(mat.matches(&Value::Int(int("-5"))));
        assert!(mat.matches(&Value::Int(int("100"))));
        assert!(!mat.matches(&Value::Null(ValueNull)));
        assert!(!mat.matches(&Value::String(ValueString::new_static("hi"))));
    }

    #[test]
    fn range_lower_only() {
        let mat = m("5..");
        assert!(mat.matches(&Value::Int(int("5"))));
        assert!(mat.matches(&Value::Int(int("10"))));
        assert!(!mat.matches(&Value::Int(int("4"))));
    }

    #[test]
    fn range_upper_exclusive() {
        let mat = m("..10");
        assert!(mat.matches(&Value::Int(int("5"))));
        assert!(mat.matches(&Value::Int(int("9"))));
        assert!(!mat.matches(&Value::Int(int("10"))));
        assert!(!mat.matches(&Value::Int(int("11"))));
    }

    #[test]
    fn range_upper_inclusive() {
        let mat = m("..=10");
        assert!(mat.matches(&Value::Int(int("10"))));
        assert!(!mat.matches(&Value::Int(int("11"))));
    }

    #[test]
    fn range_both_exclusive() {
        let mat = m("5..10");
        assert!(mat.matches(&Value::Int(int("5"))));
        assert!(mat.matches(&Value::Int(int("9"))));
        assert!(!mat.matches(&Value::Int(int("4"))));
        assert!(!mat.matches(&Value::Int(int("10"))));
    }

    #[test]
    fn range_both_inclusive() {
        let mat = m("5..=10");
        assert!(mat.matches(&Value::Int(int("5"))));
        assert!(mat.matches(&Value::Int(int("10"))));
        assert!(!mat.matches(&Value::Int(int("4"))));
        assert!(!mat.matches(&Value::Int(int("11"))));
    }

    #[test]
    fn range_negative() {
        let mat = m("-5..0");
        assert!(mat.matches(&Value::Int(int("-5"))));
        assert!(mat.matches(&Value::Int(int("-1"))));
        assert!(!mat.matches(&Value::Int(int("-6"))));
        assert!(!mat.matches(&Value::Int(int("0"))));
    }

    #[test]
    fn regex_matcher() {
        let mat = m(r#"r"\d+""#);
        assert!(mat.matches(&Value::String(ValueString::new_static("123"))));
        assert!(mat.matches(&Value::String(ValueString::new_static("5"))));
        assert!(!mat.matches(&Value::String(ValueString::new_static("abc"))));
    }

    #[test]
    fn regex_nonnil() {
        let mat = m(r#"r"[^a]""#);
        assert!(!mat.matches(&Value::Int(int("5"))));
    }

    #[test]
    fn list_slice_middle() {
        let mat = m("[1, ..., 5]");
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("5")),
        ]))));
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("2")),
            Value::Int(int("3")),
            Value::Int(int("5")),
        ]))));
        assert!(
            !mat.matches(&Value::List(ValueList::from_iter(vec![Value::Int(int(
                "5"
            )),])))
        );
    }

    #[test]
    fn list_slice_prefix() {
        let mat = m("[1, 2, ...]");
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("2")),
        ]))));
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("2")),
            Value::Int(int("3")),
        ]))));
        assert!(
            !mat.matches(&Value::List(ValueList::from_iter(vec![Value::Int(int(
                "1"
            )),])))
        );
    }

    #[test]
    fn list_slice_suffix() {
        let mat = m("[..., 4, 5]");
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("4")),
            Value::Int(int("5")),
        ]))));
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("4")),
            Value::Int(int("5")),
        ]))));
        assert!(
            !mat.matches(&Value::List(ValueList::from_iter(vec![Value::Int(int(
                "5"
            )),])))
        );
    }

    #[test]
    fn list_slice_with_matcher() {
        let mat = m("[1, ..._, 5]");
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("5")),
        ]))));
        assert!(mat.matches(&Value::List(ValueList::from_iter(vec![
            Value::Int(int("1")),
            Value::Int(int("2")),
            Value::Int(int("5")),
        ]))));
    }

    #[test]
    fn map_exact() {
        let mat = m(r#"<| a: 1 |>"#);
        let map = ValueMap::new(BTreeMap::from([(
            ValueString::new_static("a"),
            Value::Int(int("1")),
        )]));
        assert!(mat.matches(&Value::Map(map)));
    }

    #[test]
    fn map_exact_rejects_extra() {
        let mat = m(r#"<| a: 1 |>"#);
        let map = ValueMap::new(BTreeMap::from([
            (ValueString::new_static("a"), Value::Int(int("1"))),
            (ValueString::new_static("b"), Value::Int(int("2"))),
        ]));
        assert!(!mat.matches(&Value::Map(map)));
    }

    #[test]
    fn map_rest_accepts_extra() {
        let mat = m(r#"<| a: 1, ... |>"#);
        let map = ValueMap::new(BTreeMap::from([
            (ValueString::new_static("a"), Value::Int(int("1"))),
            (ValueString::new_static("b"), Value::Int(int("2"))),
        ]));
        assert!(mat.matches(&Value::Map(map)));
    }

    #[test]
    fn map_rest_rejects_missing() {
        let mat = m(r#"<| a: 1, ... |>"#);
        let map = ValueMap::new(BTreeMap::from([(
            ValueString::new_static("b"),
            Value::Int(int("2")),
        )]));
        assert!(!mat.matches(&Value::Map(map)));
    }

    #[test]
    fn map_rest_with_matcher() {
        let mat = m(r#"<| a: 1, ..._ |>"#);
        let map = ValueMap::new(BTreeMap::from([(
            ValueString::new_static("a"),
            Value::Int(int("1")),
        )]));
        assert!(mat.matches(&Value::Map(map)));
    }

    #[test]
    fn errors() {
        assert!(parse_matcher(&ValueString::new_static("foo")).is_err());
        assert!(parse_matcher(&ValueString::new_static(r#"r"["#)).is_err());
    }
}
