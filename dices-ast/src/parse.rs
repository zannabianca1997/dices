//! Parsing grammar

use std::{borrow::Cow, str::FromStr};

use either::Either::{Left, Right};
use peg::{error::ParseError, str::LineCol};

use crate::{ident::IdentStr, values::*};

peg::parser! {
    grammar value_parser() for str {
        /// A `dices` serialized value
        pub rule value() -> Value
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
            = ['"'] s:escaped_string_inner() ['"'] { s }

        // --- LISTS ---

        /// A list of values
        pub rule list() -> ValueList
            = ['['] _ items:(value() ** (_ [','] _)) _ ([','] _)? [']'] {
                ValueList::from_iter(items)
            }

        // --- MAPS ---

        /// A map of strings to values
        pub rule map() -> ValueMap
            = "<|" _ kvs: (
                k: ident_or_quoted_string() _ [':'] _ v:value() {
                    (ValueString::from(k.into_owned().into_boxed_str()), v)
                }
            ) ** [','] _ ([','] _)? "|>" { kvs.into_iter().collect() }

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

// implementations of FromStr for the value types

impl FromStr for ValueNull {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        value_parser::null(s)
    }
}
impl FromStr for ValueBool {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        value_parser::boolean(s)
    }
}
impl FromStr for ValueNumber {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        value_parser::number(s)
    }
}
impl FromStr for ValueString {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        value_parser::string(s)
    }
}
impl FromStr for ValueList {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        value_parser::list(s)
    }
}
impl FromStr for ValueMap {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        value_parser::map(s)
    }
}
impl FromStr for Value {
    type Err = ParseError<LineCol>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        value_parser::value(s)
    }
}
