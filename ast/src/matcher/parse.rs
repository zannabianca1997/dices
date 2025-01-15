use std::{borrow::Cow, collections::BTreeMap, str::FromStr};

use either::Either::{Left, Right};
use peg::{error::ParseError, str::LineCol};

use crate::ident::IdentStr;
use crate::value::{Value, ValueBool, ValueList, ValueMap, ValueNull, ValueNumber, ValueString};

use super::Matcher;

peg::parser! {
    /**
            # Value matcher

            This grammar parse a value matcher, that check if a value corresponds to some requirements
        */
    pub grammar matcher() for str {

        pub rule matcher<InjectedIntrisic>() -> Matcher<InjectedIntrisic>
            = precedence! {
                a: (
                    l: matcher_list()  { Matcher::List(l) }
                    / m: matcher_map() { Matcher::Map(m)}
                    / v: value() b:( _ ".." _ b:value() {(b, false)} / _ "..=" _ b:value() {(b, true)} )? { if let Some((end, inclusive)) = b {
                        Matcher::Range { start: v, end, inclusive }
                    } else {
                        Matcher::Exact(v)
                    } }
                    / "_" { Matcher::Any }
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
        rule value<InjectedIntrisic>() -> Value<InjectedIntrisic>
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
            = n:$(['-']?['0'..='9']+) {? n.parse().or(Err("number")) }

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
        rule list<InjectedIntrisic>() -> ValueList<InjectedIntrisic>
            = "[" _ items:(value() ** (_ "," _)) _ ("," _)? "]" {
                ValueList::from_iter(items)
            }

        /// A list of matchers
        rule matcher_list<InjectedIntrisic>() -> Box<[Matcher<InjectedIntrisic>]>
            = "[" _ items:(matcher() ** (_ "," _)) _ ("," _)? "]" {
                items.into_boxed_slice()
            }

        // --- MAPS ---

        /// A map of strings to values
        rule map<InjectedIntrisic>() -> ValueMap<InjectedIntrisic>
            = "<|" _ kvs: (
                k: ident_or_quoted_string() _ ":" _ v:value() {
                    (ValueString::from(k.into_owned().into_boxed_str()), v)
                }
            ) ** (_ "," _) _ ("," _)? "|>" { kvs.into_iter().collect() }

        /// A map of strings to matchers
        rule matcher_map<InjectedIntrisic>() -> BTreeMap<Box<str>, Matcher<InjectedIntrisic>>
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
            ([' ' | '\t' | '\r' | '\n'])* {  }
        }


    }
}

impl<InjectedIntrisic> FromStr for Matcher<InjectedIntrisic> {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        matcher::matcher(s)
    }
}
