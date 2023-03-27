//! Data structures to hold the parsed throws

use std::fmt::Debug;

use rand::Rng;
use thiserror::Error;

use crate::State;

/// A set of throws
#[derive(Debug, Clone)]
pub enum Throws {
    /// Concatenate multiple throw sets
    Concat(Box<Throws>, Box<Throws>),
    /// Multiply a throwset by a number
    ///
    /// One of the two throw sets must yield a single value.
    /// Return the result of the other, with each value multiplied by the result of the first.
    Multiply(Box<Throws>, Box<Throws>),
    /// Repeat a set of throws multiple times
    ///
    /// `times` must yield a single value.
    Repeat { base: Box<Throws>, num: Box<Throws> },
    /// Keep only the highest `num` throws
    ///
    /// `num` must yield a single value.
    KeepHigh { base: Box<Throws>, num: Box<Throws> },
    /// Keep only the lowest `num` throws
    ///
    /// `num` must yield a single value.
    KeepLow { base: Box<Throws>, num: Box<Throws> },
    /// Remove the highest `num` throws
    ///
    /// `num` must yield a single value.
    RemoveHigh { base: Box<Throws>, num: Box<Throws> },
    /// Remove the lowest `num` throws
    ///
    /// `num` must yield a single value.
    RemoveLow { base: Box<Throws>, num: Box<Throws> },
    /// A throw of a dice with an arbitrary number of faces
    ///
    /// The number of faces must yield a single value, positive and non zero.
    /// Yield a single value, positive and non zero
    Dice(Box<Throws>),
    /// A single throw that returns always the same value
    Constant(i64),
    /// Sum all the throws
    ///
    /// Yield a single value.
    Sum(Box<Throws>),
    /// Result of the last throw
    LastResult,
}

/// Errors during throwing
#[derive(Debug, Error)]
pub enum ThrowsError {
    #[error(
        "* operator need at least one of the two operands to be of lenght one, instead they were of lenghts {0} and {1}"
    )]
    MultiplyShape(usize, usize),
    #[error("{0} operator need the second operand to be of lenght one, not {1}")]
    SecondOperandMustBeScalar(&'static str, usize),
    #[error("{0} operator need the second operand to be positive, not {1}")]
    SecondOperandMustBePositive(&'static str, i64),
    #[error("dice sides must be of of lenght one, not {0}")]
    DiceSidesMustBeScalar(usize),
    #[error("dice sides must be positive and non-zero, not {0}")]
    DiceSidesMustBePositiveNonZero(i64),
    #[error("Last throw symbol `#` could be used only after a successful throw")]
    NoLastThrow,
    #[error("Overflow during addition")]
    OverflowInSum,
    #[error("Overflow during multiplication")]
    OverflowInMul,
}

/// General result type
type Result<T> = std::result::Result<T, ThrowsError>;

impl Throws {
    /// Execute this throw set
    pub fn throws(&self, state: &mut State<impl Rng>) -> Result<Vec<i64>> {
        let mut buf = vec![];
        self.throws_into(state, &mut buf)?;
        Ok(buf)
    }

    /// Execute this throw set, appending to the given container
    fn throws_into(&self, state: &mut State<impl Rng>, buf: &mut impl Extend<i64>) -> Result<()> {
        use ThrowsError::*;
        match self {
            Throws::Concat(a, b) => {
                a.throws_into(state, buf)?;
                b.throws_into(state, buf)?;
            }
            Throws::Multiply(a, b) => {
                let a = a.throws(state)?;
                let b = b.throws(state)?;
                match (a, b) {
                    (a, b) if a.len() == 1 && b.len() == 1 => {
                        buf.extend_one(i64::checked_mul(a[0], b[0]).ok_or(OverflowInMul)?)
                    }
                    (a, b) | (b, a) if b.len() == 1 => {
                        for v in a {
                            buf.extend_one(i64::checked_mul(v, b[0]).ok_or(OverflowInMul)?)
                        }
                    }
                    (a, b) => return Err(MultiplyShape(a.len(), b.len())),
                }
            }
            Throws::Repeat { base, num } => {
                let num = num.throws(state)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("^", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("^", num));
                }
                let num = num as usize;
                for _ in 0..num {
                    base.throws_into(state, buf)?;
                }
            }
            Throws::KeepHigh { base, num } => {
                let num = num.throws(state)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("kh", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("kh", num));
                }
                let num = num as usize;

                let mut res = base.throws(state)?;
                res.sort();
                buf.extend(res.into_iter().rev().take(num))
            }
            Throws::KeepLow { base, num } => {
                let num = num.throws(state)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("kl", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("kl", num));
                }
                let num = num as usize;

                let mut res = base.throws(state)?;
                res.sort();
                buf.extend(res.into_iter().take(num))
            }
            Throws::RemoveHigh { base, num } => {
                let num = num.throws(state)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("kh", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("kh", num));
                }
                let num = num as usize;

                let mut res = base.throws(state)?;
                res.sort();
                buf.extend(res.into_iter().rev().skip(num))
            }
            Throws::RemoveLow { base, num } => {
                let num = num.throws(state)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("kh", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("kh", num));
                }
                let num = num as usize;

                let mut res = base.throws(state)?;
                res.sort();
                buf.extend(res.into_iter().skip(num))
            }
            Throws::Dice(sides) => {
                let sides = sides.throws(state)?;
                if sides.len() != 1 {
                    return Err(DiceSidesMustBeScalar(sides.len()));
                }
                let sides = sides[0];
                if sides <= 0 {
                    return Err(DiceSidesMustBePositiveNonZero(sides));
                }
                let sides = sides as usize;

                buf.extend_one(state.rng.gen_range(1..=sides) as i64)
            }
            Throws::Constant(v) => buf.extend_one(*v),
            Throws::Sum(throws) => buf.extend_one(
                throws
                    .throws(state)?
                    .into_iter()
                    .try_reduce(|a, b| i64::checked_add(a, b))
                    .ok_or(OverflowInSum)?
                    .unwrap_or(0),
            ),
            Throws::LastResult => {
                if let Some(vec) = &state.last_res {
                    buf.extend(vec.iter().copied())
                } else {
                    return Err(NoLastThrow);
                }
            }
        }
        Ok(())
    }
}
