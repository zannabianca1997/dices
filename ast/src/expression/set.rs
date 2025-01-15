//! set and let expressions

use crate::ident::IdentStr;

use super::Expression;

/// An `=` expression

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
pub struct ExpressionSet<InjectedIntrisic> {
    /// Where the value must be put
    pub receiver: Receiver<InjectedIntrisic>,
    /// The value to set
    pub value: Box<Expression<InjectedIntrisic>>,
}

/// The lhs of a `=` expression

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
pub enum Receiver<InjectedIntrisic> {
    /// `_` receiver: throw away its value
    Ignore,
    /// Set a variable
    Set(MemberReceiver<InjectedIntrisic>),
    /// Let a new variable
    Let(Box<IdentStr>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
pub struct MemberReceiver<InjectedIntrisic> {
    /// The variable receiving the value
    pub root: Box<IdentStr>,
    /// The multiple indices
    pub indices: Vec<Expression<InjectedIntrisic>>,
}
impl<II> MemberReceiver<II> {
    #[must_use]
    pub const fn new(root: Box<IdentStr>, indices: Vec<Expression<II>>) -> Self {
        Self { root, indices }
    }
}
