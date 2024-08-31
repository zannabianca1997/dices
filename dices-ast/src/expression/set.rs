//! set and let expressions

use crate::ident::IdentStr;

use super::Expression;

/// An `=` expression

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionSet {
    /// Where the value must be put
    pub receiver: Receiver,
    /// The value to set
    pub value: Box<Expression>,
}

/// The lhs of a `=` expression

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Receiver {
    /// `_` receiver: throw away its value
    Ignore,
    /// Set a variable
    Set(Box<IdentStr>),
    /// Let a new variable
    Let(Box<IdentStr>),
}
