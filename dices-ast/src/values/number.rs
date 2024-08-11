use derive_more::derive::{
    Add, AddAssign, Display, Div, DivAssign, FromStr, Mul, MulAssign, Neg, Sub, SubAssign,
};

use super::list::ValueList;

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

    pub fn to_list(self) -> Result<ValueList, super::ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}
