//! Data structures to hold the parsed throws

use std::fmt::Debug;

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
    Repeat {
        base: Box<Throws>,
        times: Box<Throws>,
    },
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
