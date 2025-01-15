use derive_more::derive::{Deref, DerefMut, Display, From, Into};

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
    Deref,
    DerefMut,
)]
#[cfg_attr(feature = "bincode", derive(bincode::Decode, bincode::Encode,))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValueBool(bool);
impl ValueBool {
    pub const TRUE: Self = ValueBool(true);
    pub const FALSE: Self = ValueBool(false);

    pub fn to_number(self) -> Result<ValueNumber, ToNumberError> {
        Ok(if self.0 {
            ValueNumber::from(1)
        } else {
            ValueNumber::ZERO
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
        allocator.text(if self.0 { "true" } else { "false" })
    }
}
