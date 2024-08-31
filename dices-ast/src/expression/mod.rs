//! Type containing a `dices` expression
use derive_more::derive::From;

use crate::values::Value;

pub use bin_ops::ExpressionBinOp;
pub use call::ExpressionCall;
pub use closure::ExpressionClosure;
pub use list::ExpressionList;
pub use map::ExpressionMap;
pub use member_access::ExpressionMemberAccess;
pub use ref_::ExpressionRef;
pub use scope::ExpressionScope;
pub use set::{ExpressionSet, Receiver};
pub use un_ops::ExpressionUnOp;

pub mod bin_ops;
pub mod call;
pub mod closure;
pub mod list;
pub mod map;
pub mod ref_;
pub mod scope;
pub mod set;
pub mod un_ops;
pub mod member_access {
    //! Expression to read the members of a composite

    use super::Expression;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    /// Access a member of a map or a list
    pub struct ExpressionMemberAccess {
        pub accessed: Box<Expression>,
        pub index: Box<Expression>,
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

    /// Member access
    MemberAccess(ExpressionMemberAccess),

    /// Scoping expression
    Scope(ExpressionScope),

    /// Set expression
    Set(ExpressionSet),
    /// Ref expression
    Ref(ExpressionRef),
}
