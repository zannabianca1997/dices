use crate::{expr::Expr, identifier::Identifier};

/// Member access
///
/// This is generated both on explicit indexing `a["b"]` and with dot notation
/// `a.b`, `a.0`, `a."b"`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemberAccessLhs {
    pub container: Box<Lhs>,
    pub index: Expr,
}

/// Assignment left hand side
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lhs {
    /// Assign to a variable
    Variable(Identifier),
    /// Assign to a member of a variable
    MemberAccess(MemberAccessLhs),
}

/// Assign statement
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AssignStatement {
    /// Let statement
    ///
    /// Create a variable, shadowing others
    Let { lhs: Identifier, rhs: Expr },
    /// Set statement
    ///
    /// This has a more general receiver (e.g member access)
    Set { lhs: Lhs, rhs: Expr },
}
