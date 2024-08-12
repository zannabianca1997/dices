//! Type containing a `dices` expression
use derive_more::derive::From;

use crate::values::Value;

pub use bin_ops::ExpressionBinOp;
pub use call::ExpressionCall;
pub use closure::ExpressionClosure;
pub use list::ExpressionList;
pub use map::ExpressionMap;
pub use scope::ExpressionScope;
pub use un_ops::ExpressionUnOp;

pub mod bin_ops;
pub mod call;
pub mod closure;
pub mod list;
pub mod map;
pub mod scope;
pub mod un_ops;

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
}
