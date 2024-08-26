//! Type containing a `dices` expression
use derive_more::derive::From;

use crate::values::Value;

pub use bin_ops::ExpressionBinOp;
pub use call::ExpressionCall;
pub use closure::ExpressionClosure;
pub use list::ExpressionList;
pub use map::ExpressionMap;
pub use ref_::ExpressionRef;
pub use scope::ExpressionScope;
pub use set::{ExpressionSet, Receiver};
pub use un_ops::ExpressionUnOp;

pub mod bin_ops;
pub mod call;
pub mod closure;
pub mod list;
pub mod map;
pub mod scope;
pub mod un_ops;
pub mod set {
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
}
pub mod ref_ {
    //! ref expressions

    use crate::ident::IdentStr;

    /// An expression referencing a variable
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ExpressionRef {
        /// The name of the variable
        pub name: Box<IdentStr>,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub enum Expression {
    /// Expression returning a constant value
    Const(Value),

    /// List literal
    List(ExpressionList),
    /// Map literal
    Map(ExpressionMap),

    /// Closure literal
    Closure(ExpressionClosure),

    /// Expression with a single operatorand
    UnOp(ExpressionUnOp),
    /// Expression with two operatorands
    BinOp(ExpressionBinOp),

    /// Call expression
    Call(ExpressionCall),

    /// Scoping expression
    Scope(ExpressionScope),

    /// Set expression
    Set(ExpressionSet),
    /// Ref expression
    Ref(ExpressionRef),
}
