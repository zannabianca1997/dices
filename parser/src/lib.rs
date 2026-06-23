#![doc = include_str!("../README.md")]

use std::str::FromStr;

use dices_ast::expr::scope::ScopeInner;
use dices_values::{
    int::ValueInt,
    string::{EscapeError, ValueString},
};
use pest::Parser;
use pest_derive::Parser;
use snafu::{ResultExt, Snafu};

pub(crate) mod identifier;
pub(crate) mod literal;

pub(crate) mod expr;
pub(crate) mod statement;

pub mod value;

#[derive(Parser)]
#[grammar = "ast.pest"]
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

pub fn parse_scope_inner(input: &ValueString) -> Result<ScopeInner, ParseError> {
    let raw = input.as_str();
    let mut pairs = Grammar::parse(Rule::main, raw).context(PestSnafu)?;
    let scope_inner_pair = pairs.next().unwrap();
    expr::scope::build_scope_inner(scope_inner_pair, input)
}

fn unexpected_rule(r: Rule) -> ! {
    unreachable!("Unexpected rule {r:?}")
}

#[cfg(test)]
pub(crate) mod tests {
    use dices_ast::expr::scope::ScopeInner;
    use dices_values::string::ValueString;

    use crate::parse_scope_inner;

    pub fn parse(input: &'static str) -> ScopeInner {
        parse_scope_inner(&ValueString::new_static(input))
            .expect("parse_scope_inner should succeed for a valid input")
    }

    pub fn parse_err(input: &'static str) -> crate::ParseError {
        parse_scope_inner(&ValueString::new_static(input)).unwrap_err()
    }

    // Error cases

    #[test]
    fn empty_input_yields_empty_scope_inner() {
        let inner = parse("");
        assert!(inner.statements.is_empty());
        assert!(inner.expr.is_none());
    }

    #[test]
    fn error_garbage() {
        assert!(parse_err("foo bar").is_pest());
    }
}
