//! Type containing a `dices` expression

use derive_more::derive::From;

use crate::values::Value;

pub mod bin_ops;
pub mod call;
pub mod closure;
pub mod list;
pub mod map;
pub mod un_ops;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
pub enum Expression {
    /// Expression returning a constant value
    Const(Value),

    /// List literal
    List(list::ExpressionList),
    /// Map literal
    Map(map::ExpressionMap),

    /// Closure literal
    Closure(closure::ExpressionClosure),

    /// Expression with a single operatorand
    UnOp(un_ops::ExpressionUnOp),
    /// Expression with two operatorands
    BinOp(bin_ops::ExpressionBinOp),

    /// Call expression
    Call(call::ExpressionCall),
}
