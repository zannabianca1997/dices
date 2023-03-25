use std::str::FromStr;

use pest::{iterators::Pair, pratt_parser::PrattParser, Parser};
use pest_derive::Parser;

use super::throws::Throws;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub(super) struct ThrowsParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        // Operators are, in reverse precedence order
        // - -> unary
        // d -> can be unary or binary infix
        // ^ -> binary infix
        // *, kh, kl, rh, rl -> binary infix
        // +, - -> binary infix
        PrattParser::new()
            .op(Op::infix(concat, Left) | Op::infix(minus, Left))
            .op(Op::infix(multiply, Left) | Op::infix(kh, Left) | Op::infix(kl, Left) | Op::infix(rh, Left) | Op::infix(rl, Left))
            .op(Op::infix(repeats, Left))
            .op(Op::infix(dice,Left))
    };
}

/// Error in parsing throws
pub type Error = pest::error::Error<Rule>;

impl FromStr for Throws {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ThrowsParser::parse(Rule::throwsargs, s).map(|mut pairs| pairs.next().unwrap().into())
    }
}

impl From<Pair<'_, Rule>> for Throws {
    fn from(value: Pair<'_, Rule>) -> Self {
        debug_assert_eq!(value.as_rule(), Rule::throws);

        PRATT_PARSER
            .map_primary(|a| {
                let mut pairs = a.into_inner().rev();
                let inner_atom = pairs.next().unwrap();
                let mut throw = match inner_atom.as_rule() {
                    Rule::lit => Self::Constant(
                        inner_atom
                            .as_str()
                            .parse()
                            .expect("`lit` should always be parseable as i64"),
                    ),
                    Rule::summed => {
                        Self::Sum(Box::new(inner_atom.into_inner().next().unwrap().into()))
                    }
                    Rule::throws => inner_atom.into(),
                    r => unreachable!("Rule {r:?} shouldn't appear as a `throwsexpr` atom"),
                };
                for op in pairs {
                    throw = match op.as_rule() {
                        Rule::dice => Self::Dice(Box::new(throw)),
                        Rule::minus => {
                            Self::Multiply(Box::new(Self::Constant(-1)), Box::new(throw))
                        }
                        r => unreachable!(
                            "Rule {r:?} shouldn't appear as a `throwsexpr` unary operation"
                        ),
                    }
                }
                throw
            })
            .map_infix(|a, op, b| match op.as_rule() {
                Rule::concat => Self::Concat(Box::new(a), Box::new(b)),
                Rule::minus => Self::Concat(
                    Box::new(a),
                    Box::new(Self::Multiply(Box::new(Self::Constant(-1)), Box::new(b))),
                ),
                Rule::multiply => Self::Multiply(Box::new(a), Box::new(b)),
                Rule::repeats => Self::Repeat {
                    base: Box::new(a),
                    num: Box::new(b),
                },
                Rule::kh => Self::KeepHigh {
                    base: Box::new(a),
                    num: Box::new(b),
                },
                Rule::kl => Self::KeepLow {
                    base: Box::new(a),
                    num: Box::new(b),
                },
                Rule::rh => Self::RemoveHigh {
                    base: Box::new(a),
                    num: Box::new(b),
                },
                Rule::rl => Self::RemoveLow {
                    base: Box::new(a),
                    num: Box::new(b),
                },
                Rule::dice => Self::Repeat {
                    base: Box::new(Self::Dice(Box::new(b))),
                    num: Box::new(a),
                },
                r => unreachable!("Rule {r:?} shouldn't appear as a `throwsexpr` binary operation"),
            })
            .parse(value.into_inner())
    }
}
