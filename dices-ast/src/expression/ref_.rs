//! ref expressions

use crate::ident::IdentStr;

/// An expression referencing a variable
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionRef {
    /// The name of the variable
    pub name: Box<IdentStr>,
}
