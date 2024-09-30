use std::iter::Step;

use derive_more::derive::{Display, From, Into};

use super::{list::ValueList, number::ValueNumber, ToNumberError};

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
    // conversions
    From,
    Into,
)]
#[cfg_attr(feature = "bincode", derive(bincode::Decode, bincode::Encode,))]
pub struct ValueBool(bool);
impl ValueBool {
    pub const TRUE: Self = ValueBool(true);
    pub const FALSE: Self = ValueBool(false);

    pub fn to_number(self) -> Result<ValueNumber, ToNumberError> {
        Ok(match self.0 {
            true => Step::forward(ValueNumber::ZERO, 1),
            false => ValueNumber::ZERO,
        })
    }

    pub fn to_list<InjectedIntrisic>(
        self,
    ) -> Result<super::list::ValueList<InjectedIntrisic>, super::ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}

#[cfg(feature = "pretty")]
impl<'a, D, A> pretty::Pretty<'a, D, A> for &'a ValueBool
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator.text(match self.0 {
            true => "true",
            false => "false",
        })
    }
}
