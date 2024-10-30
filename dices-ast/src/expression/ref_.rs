//! ref expressions

use crate::ident::IdentStr;

/// An expression referencing a variable
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "bincode", derive(bincode::Decode, bincode::Encode,))]
pub struct ExpressionRef {
    /// The name of the variable
    pub name: Box<IdentStr>,
}
