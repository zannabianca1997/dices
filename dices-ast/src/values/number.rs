use derive_more::derive::{
    Add, AddAssign, Display, Div, DivAssign, From, Into, Mul, MulAssign, Neg, Sub, SubAssign,
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
    // conversion
    From,
    Into,
)]
pub struct ValueNumber(i64);

impl ValueNumber {
    pub const ZERO: Self = ValueNumber(0);
    pub const ONE: Self = ValueNumber(1);

    pub fn to_number(self) -> Result<ValueNumber, super::ToNumberError> {
        Ok(self)
    }

    pub fn to_list<InjectedIntrisic>(
        self,
    ) -> Result<ValueList<InjectedIntrisic>, super::ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}

#[cfg(feature = "pretty")]
impl<'a, D, A> pretty::Pretty<'a, D, A> for &'a ValueNumber
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator.text(self.to_string())
    }
}
