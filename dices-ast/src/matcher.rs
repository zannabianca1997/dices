use crate::ident::IdentStr;
use crate::values::*;
use either::Either::{Left, Right};
use peg::{error::ParseError, str::LineCol};
use std::{borrow::Cow, collections::BTreeMap, str::FromStr};

#[derive(Debug, Clone)]
pub enum Matcher {
    Exact(Value),
    List(Box<[Matcher]>),
    Map(BTreeMap<Box<str>, Matcher>),
    Range {
        start: Value,
        end: Value,
        inclusive: bool,
    },
    And(Box<[Matcher; 2]>),
    Or(Box<[Matcher; 2]>),
    Not(Box<Matcher>),
}
impl Matcher {
    pub fn is_match(&self, v: &Value) -> bool {
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
            Matcher::List(box matchers) => {
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
                matchers.iter().all(|(box name, matcher)| {
                    let Some(value) = values.get(name) else {
                        return false;
                    };
                    matcher.is_match(value)
                })
                // we do not have to check for orfan values, because the maps have the same size.
            }
            Matcher::And(box [a, b]) => a.is_match(v) && b.is_match(v),
            Matcher::Or(box [a, b]) => a.is_match(v) || b.is_match(v),
            Matcher::Not(box a) => !a.is_match(v),
        }
    }
}

peg::parser! {
    /**
        # Value matcher

        This grammar parse a value matcher, that check if a value corresponds to some requirements
    */
    pub grammar matcher() for str {

        pub rule matcher() -> Matcher
            = precedence! {
                a: (
                    l: matcher_list()  { Matcher::List(l) }
                    / m: matcher_map() { Matcher::Map(m)}
                    / v: value() b:( _ ".." _ b:value() {(b, false)} / _ "..=" _ b:value() {(b, true)} )? { if let Some((end, inclusive)) = b {
                        Matcher::Range { start: v, end, inclusive }
                    } else {
                        Matcher::Exact(v)
                    } }
                    / "(" _ m:matcher() _ ")" { m }
                ) {a}
                --
                a: (@) _ "||" _ b:@ { Matcher::Or(Box::new([a,b]))  }
                --
                a: (@) _ "&&" _ b:@ { Matcher::And(Box::new([a,b])) }
                --
                "!" _ a:@ { Matcher::Not(Box::new(a)) }
            }

        /// A `dices` serialized value
        rule value() -> Value
            = v: null()    { v.into() }
            / v: boolean() { v.into() }
            / v: number()  { v.into() }
            / v: string()  { v.into() }
            / v: list()    { v.into() }
            / v: map()     { v.into() }

        // --- SCALARS ---

        /// A null value
        pub rule null() -> ValueNull
            = "null"  { ValueNull }

        /// A boolean value
        pub rule boolean() -> ValueBool
            = "true"  { ValueBool::TRUE  }
            / "false" { ValueBool::FALSE }

        /// A signed number
        pub rule number() -> ValueNumber
            = n:$(['-']?['0'..='9']+) {? n.parse::<i64>().map(Into::into).or(Err("number")) }

        /// A quoted string value
        pub rule string() -> ValueString
            = s:quoted_string() { ValueString::from(s.into_owned().into_boxed_str()) }

        // --- STRING QUOTING AND ESCAPING ---

        /// The inner part of a string literal
        rule escaped_string_inner() -> std::borrow::Cow<'input,str>
            = parts: (
                lit: $( [^ '"' | '\\']+ ) { Left(lit) }
                / "\\" escape:(
                        "n" {'\n'}
                        / "r" {'\r'}
                        / "t" {'\t'}
                        / "0" {'\0'}
                        / "\\" {'\\'}
                        / "\'" {'\''}
                        / "\"" {'"'}
                        / hex:(
                        "x" hex: $(['0'..='7']['a'..='f'|'A'..='F'|'0'..='9']) {hex}
                        / "u{" hex: $(['a'..='f'|'A'..='F'|'0'..='9']*<1,6>) "}" {hex}) {?
                                char::from_u32(u32::from_str_radix(hex, 16).unwrap())
                                .ok_or("unicode codepoint")
                        }
                        / expected!("escape code")

                ) {Right(escape)}
                )*  {
                    if let [Left(part)] = &*parts {
                        Cow::Borrowed(*part)
                    } else {
                        // some escapes remains. We need to build a string
                        let mut buf = std::string::String::with_capacity(
                            parts.iter().map(|p| match p {
                                Left(s) => s.len(),
                                Right(c) => c.len_utf8(),
                            }).sum()
                        );
                        for p in parts {
                            match p {
                                Left(s) => buf.push_str(s),
                                Right(ch) => buf.push(ch),
                            }
                        }
                        Cow::Owned(buf)
                    }
                }

        /// A string literal
        rule quoted_string() -> std::borrow::Cow<'input,str>
            = "\"" s:escaped_string_inner() "\"" { s }

        // --- LISTS ---

        /// A list of values
        rule list() -> ValueList
            = "[" _ items:(value() ** (_ "," _)) _ ("," _)? "]" {
                ValueList::from_iter(items)
            }

        /// A list of matchers
        rule matcher_list() -> Box<[Matcher]>
            = "[" _ items:(matcher() ** (_ "," _)) _ ("," _)? "]" {
                items.into_boxed_slice()
            }

        // --- MAPS ---

        /// A map of strings to values
        rule map() -> ValueMap
            = "<|" _ kvs: (
                k: ident_or_quoted_string() _ ":" _ v:value() {
                    (ValueString::from(k.into_owned().into_boxed_str()), v)
                }
            ) ** (_ "," _) _ ("," _)? "|>" { kvs.into_iter().collect() }

        /// A map of strings to matchers
        rule matcher_map() -> BTreeMap<Box<str>, Matcher>
            = "<|" _ kvs: (
                k: ident_or_quoted_string() _ ":" _ v:matcher() {
                    (k.into_owned().into_boxed_str(), v)
                }
            ) ** (_ "," _) _ ("," _)? "|>" { kvs.into_iter().collect() }

        /// An identifier
        rule ident() -> &'input IdentStr
            = i:$(
                (['a'..='z'|'A'..='Z'] / ['_']+ ['0'..='9'|'a'..='z'|'A'..='Z'])
                ['0'..='9'|'a'..='z'|'A'..='Z'|'_']*
            ) {? IdentStr::new(i).ok_or("identifier") }


        /// Either a identifier or a wuoted string literal
        rule ident_or_quoted_string() -> std::borrow::Cow<'input,str>
        = i: ident()         { Cow::Borrowed(&**i) }
        / s: quoted_string() { s }

        // --- WHITESPACE ---

        /// Parse whitespaces, discarding them
        rule _ -> ()
        = quiet!{
            ([' ' | '\t' | '\r' | '\n'])* { () }
        }


    }
}

impl FromStr for Matcher {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        matcher::matcher(s)
    }
}
