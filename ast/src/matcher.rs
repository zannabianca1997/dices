use crate::value::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Hash)]
pub enum Matcher<InjectedIntrisic> {
    Exact(Value<InjectedIntrisic>),
    List(Box<[Matcher<InjectedIntrisic>]>),
    Map(BTreeMap<Box<str>, Matcher<InjectedIntrisic>>),
    Range {
        start: Value<InjectedIntrisic>,
        end: Value<InjectedIntrisic>,
        inclusive: bool,
    },
    And(Box<[Matcher<InjectedIntrisic>; 2]>),
    Or(Box<[Matcher<InjectedIntrisic>; 2]>),
    Not(Box<Matcher<InjectedIntrisic>>),
    Any,
    None,
}
impl<InjectedIntrisic> Matcher<InjectedIntrisic> {
    pub fn is_match(&self, v: &Value<InjectedIntrisic>) -> bool
    where
        InjectedIntrisic: Eq + Ord,
    {
        match self {
            Matcher::Exact(t) => v == t,
            Matcher::Range {
                start,
                end,
                inclusive: false,
            } => start <= v && v < end,
            Matcher::Range {
                start,
                end,
                inclusive: true,
            } => start <= v && v <= end,
            Matcher::List(matchers) => {
                let Value::List(values) = v else {
                    return false;
                };
                let values = &**values;
                if values.len() != matchers.len() {
                    return false;
                }
                matchers.iter().zip(values).all(|(m, v)| m.is_match(v))
            }
            Matcher::Map(matchers) => {
                let Value::Map(values) = v else {
                    return false;
                };
                // check the map have the same size
                if values.len() != matchers.len() {
                    return false;
                }
                // check that all matchers have their match
                matchers.iter().all(|(name, matcher)| {
                    let Some(value) = values.get(name) else {
                        return false;
                    };
                    matcher.is_match(value)
                })
                // we do not have to check for orfan values, because the maps have the same size.
            }
            Matcher::And(ops) => ops[0].is_match(v) && ops[1].is_match(v),
            Matcher::Or(ops) => ops[0].is_match(v) || ops[1].is_match(v),
            Matcher::Not(a) => !a.is_match(v),
            Matcher::Any => true,
            Matcher::None => false,
        }
    }
}

#[cfg(feature = "parse_matcher")]
mod parse;
