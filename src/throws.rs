//! Data structures to hold the parsed throws

use std::fmt::Debug;

use rand::Rng;
use thiserror::Error;

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
}

/// General result type
type Result<T> = std::result::Result<T, ThrowsError>;

impl Throws {
    /// Execute this throw set
    pub fn throws(&self, rng: &mut impl Rng) -> Result<Vec<i64>> {
        let mut buf = vec![];
        self.throws_into(rng, &mut buf)?;
        Ok(buf)
    }

    /// Execute this throw set, appending to the given container
    fn throws_into(&self, rng: &mut impl Rng, buf: &mut impl Extend<i64>) -> Result<()> {
        use ThrowsError::*;
        match self {
            Throws::Concat(a, b) => {
                a.throws_into(rng, buf)?;
                b.throws_into(rng, buf)?;
            }
            Throws::Multiply(a, b) => {
                let a = a.throws(rng)?;
                let b = b.throws(rng)?;
                match (a.len(), b.len()) {
                    (1, 1) => buf.extend_one(a[0] * b[0]),
                    (1, _) => buf.extend(b.into_iter().map(|b| b * a[0])),
                    (_, 1) => buf.extend(a.into_iter().map(|a| a * b[0])),
                    (a, b) => return Err(MultiplyShape(a, b)),
                }
            }
            Throws::Repeat { base, num } => {
                let num = num.throws(rng)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("^", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("^", num));
                }
                let num = num as usize;
                for _ in 0..num {
                    base.throws_into(rng, buf)?;
                }
            }
            Throws::KeepHigh { base, num } => {
                let num = num.throws(rng)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("kh", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("kh", num));
                }
                let num = num as usize;

                let mut res = base.throws(rng)?;
                res.sort();
                buf.extend(res.into_iter().rev().take(num))
            }
            Throws::KeepLow { base, num } => {
                let num = num.throws(rng)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("kl", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("kl", num));
                }
                let num = num as usize;

                let mut res = base.throws(rng)?;
                res.sort();
                buf.extend(res.into_iter().take(num))
            }
            Throws::RemoveHigh { base, num } => {
                let num = num.throws(rng)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("kh", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("kh", num));
                }
                let num = num as usize;

                let mut res = base.throws(rng)?;
                res.sort();
                buf.extend(res.into_iter().rev().skip(num))
            }
            Throws::RemoveLow { base, num } => {
                let num = num.throws(rng)?;
                if num.len() != 1 {
                    return Err(SecondOperandMustBeScalar("kh", num.len()));
                }
                let num = num[0];
                if num < 0 {
                    return Err(SecondOperandMustBePositive("kh", num));
                }
                let num = num as usize;

                let mut res = base.throws(rng)?;
                res.sort();
                buf.extend(res.into_iter().skip(num))
            }
            Throws::Dice(sides) => {
                let sides = sides.throws(rng)?;
                if sides.len() != 1 {
                    return Err(DiceSidesMustBeScalar(sides.len()));
                }
                let sides = sides[0];
                if sides <= 0 {
                    return Err(DiceSidesMustBePositiveNonZero(sides));
                }
                let sides = sides as usize;

                buf.extend_one(rng.gen_range(1..=sides) as i64)
            }
            Throws::Constant(v) => buf.extend_one(*v),
            Throws::Sum(throws) => buf.extend_one(throws.throws(rng)?.into_iter().sum()),
        }
        Ok(())
    }
}
