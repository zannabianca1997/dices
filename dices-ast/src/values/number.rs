use derive_more::derive::{
    Add, AddAssign, Display, Div, DivAssign, FromStr, Mul, MulAssign, Neg, Sub, SubAssign,
};

/// A signed integer value
#[derive(
    // display helper
    Debug,
    Display,
    // cloning
    Clone,
    Copy,
    // comparisons
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    // number operations
    Add,
    AddAssign,
    Sub,
    SubAssign,
    Mul,
    MulAssign,
    Div,
    DivAssign,
    Neg,
    FromStr,
)]
pub struct ValueNumber(i64);

impl ValueNumber {
    pub const ZERO: Self = ValueNumber(0);
    pub const ONE: Self = ValueNumber(1);

    pub fn to_number(self) -> Result<ValueNumber, super::ToNumberError> {
        Ok(self)
    }
}
