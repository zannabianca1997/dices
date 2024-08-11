#![doc = include_str!("../README.md")]

pub mod fmt;
pub mod ident;
pub mod intrisics;
pub mod parse;
pub mod values;
pub mod expression {
    //! Type containing a `dices` expression

    use crate::values::Value;

    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub enum Expression {
        /// Expression returning a constant value
        Const(Value),
    }
}
