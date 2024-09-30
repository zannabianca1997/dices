//! Type containing a `dices` expression
use derive_more::derive::From;

use crate::value::Value;

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
pub mod member_access;
pub mod ref_;
pub mod scope;
pub mod set;
pub mod un_ops;

#[cfg(feature = "parse_expression")]
mod parse;
#[cfg(feature = "parse_expression")]
pub use parse::{parse_file, Error as ParseError};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, From)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode,),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr + 'static")
)]
pub enum Expression<InjectedIntrisic> {
    /// Expression returning a constant value
    Const(Value<InjectedIntrisic>),

    /// List literal
    List(ExpressionList<InjectedIntrisic>),
    /// Map literal
    Map(ExpressionMap<InjectedIntrisic>),

    /// Closure literal
    Closure(ExpressionClosure<InjectedIntrisic>),

    /// Expression with a single operatorand
    UnOp(ExpressionUnOp<InjectedIntrisic>),
    /// Expression with two operatorands
    BinOp(ExpressionBinOp<InjectedIntrisic>),

    /// Call expression
    Call(ExpressionCall<InjectedIntrisic>),

    /// Member access
    MemberAccess(ExpressionMemberAccess<InjectedIntrisic>),

    /// Scoping expression
    Scope(ExpressionScope<InjectedIntrisic>),

    /// Set expression
    Set(ExpressionSet<InjectedIntrisic>),
    /// Ref expression
    Ref(ExpressionRef),
}
