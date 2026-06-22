use dices_ast::identifier::Identifier;
use dices_values::string::ValueString;
use pest::iterators::Pair;

use crate::{ParseError, Rule};

/// Parse an identifier out of a [`Rule::identifier`] pair.
///
/// The identifier is sliced directly out of `input`, sharing its backing
/// allocation instead of copying the text into a fresh string.
pub(crate) fn parse_identifier(
    pair: Pair<Rule>,
    input: &ValueString,
) -> Result<Identifier, ParseError> {
    let span = pair.as_span();
    // `identifier` is an atomic rule, so the whole span is the identifier: no
    // trimming needed. Identifiers are ASCII, so the range is always valid.
    let text = input.slice(span.start()..span.end()).unwrap();
    Identifier::new(text).map_err(|text| ParseError::InvalidIdentifier { text })
}

#[cfg(test)]
pub(crate) fn ident(s: &'static str) -> Identifier {
    Identifier::new(ValueString::new_static(s)).unwrap()
}
