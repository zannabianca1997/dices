use crate::{expr::Expr, identifier::Identifier};

/// Assignment left hand side
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Lhs {
    /// Assign to a variable
    Variable(Identifier),
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
