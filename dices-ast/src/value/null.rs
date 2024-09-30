//! The `null` value

use std::fmt::Display;

use super::{ToNumberError, ValueList};

#[derive(
    // display helper
    Debug,
    // cloning
    Clone,
    // comparisons
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
)]
#[cfg_attr(feature = "bincode", derive(bincode::Decode, bincode::Encode,))]
pub struct ValueNull;
impl ValueNull {
    pub fn to_number(self) -> Result<super::ValueNumber, super::ToNumberError> {
        Err(ToNumberError::InvalidNull)
    }
    pub fn to_list<InjectedIntrisic>(
        self,
    ) -> Result<super::ValueList<InjectedIntrisic>, super::ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}

impl Display for ValueNull {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}

#[cfg(feature = "pretty")]
impl<'a, D, A> pretty::Pretty<'a, D, A> for &'a ValueNull
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator.text("null")
    }
}
