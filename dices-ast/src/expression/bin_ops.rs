//! binary operations

use derive_more::derive::Display;

use super::Expression;

/// An unary operator
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BinOp {
    /// `+`: Sum lists and maps, recursive
    Add,
    /// `-`: Negate. Distribute inside lists and maps
    Sub,
    /// `~`: Join list and maps
    Join,
    /// `^`: Repeat an operation, building a list
    Repeat,
    /// `*`: multiply, distributing scalars over lists or maps
    Mult,
    /// `%`: remainder, distributing scalars over lists or maps
    Rem,
    /// `/`: divide, distributing scalars over lists or maps
    Div,
    /// `kh`: keep the highest n values of a list or map
    KeepHigh,
    /// `kl`: keep the lowest n values of a list or map
    KeepLow,
    /// `rh`: keep the highest n values of a list or map
    RemoveHigh,
    /// `rl`: keep the lowest n values of a list or map
    RemoveLow,
}

impl BinOp {
    /// Return the evaluation order.
    /// Return `None` if the operator has a custom way of evaluate the operands
    #[inline(always)]
    pub const fn eval_order(&self) -> Option<EvalOrder> {
        match self {
            BinOp::Add | BinOp::Sub | BinOp::Join | BinOp::Mult | BinOp::Rem | BinOp::Div => {
                Some(EvalOrder::AB)
            }
            BinOp::Repeat => None,
            BinOp::KeepHigh | BinOp::KeepLow | BinOp::RemoveHigh | BinOp::RemoveLow => {
                Some(EvalOrder::BA)
            }
        }
    }
}

/// An expression made with an unary operator
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionBinOp<InjectedIntrisic> {
    pub op: BinOp,
    pub expressions: Box<[Expression<InjectedIntrisic>; 2]>,
}

impl<InjectedIntrisic> ExpressionBinOp<InjectedIntrisic> {
    pub fn new(
        op: BinOp,
        a: Expression<InjectedIntrisic>,
        b: Expression<InjectedIntrisic>,
    ) -> Self {
        Self {
            op,
            expressions: Box::new([a, b]),
        }
    }
}
/// Order of evaluation of the operands
#[derive(Debug, Clone, Copy)]
pub enum EvalOrder {
    /// first LHS, then RHS
    AB,
    /// first RHS, then LHS
    BA,
}
