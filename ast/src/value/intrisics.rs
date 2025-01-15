use std::fmt::Display;

use derive_more::derive::{From, Into};

use crate::intrisics::{InjectedIntr, Intrisic, NoInjectedIntrisics};

use super::{ToListError, ToNumberError, ValueList, ValueNumber};

/// Value representing an intrisic
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
    // conversion
    From,
    Into,
)]
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode),
    bincode(bounds = "Injected: crate::intrisics::InjectedIntr")
)]
pub struct ValueIntrisic<Injected>(pub Intrisic<Injected>);

impl<Injected: InjectedIntr> Display for ValueIntrisic<Injected> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<intrisic `{}`>", self.0.name())
    }
}

impl<Injected> ValueIntrisic<Injected> {
    pub const fn to_number(&self) -> Result<ValueNumber, ToNumberError> {
        Err(ToNumberError::Intrisic)
    }
    pub fn to_list(self) -> Result<ValueList<Injected>, ToListError> {
        Ok(ValueList::from_iter([self.into()]))
    }
}
impl ValueIntrisic<NoInjectedIntrisics> {
    pub const fn with_arbitrary_injected_intrisics<II>(self) -> ValueIntrisic<II> {
        ValueIntrisic(self.0.with_arbitrary_injected_intrisics())
    }
}

#[cfg(feature = "pretty")]
impl<'a, D, A, II> pretty::Pretty<'a, D, A> for &'a ValueIntrisic<II>
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
    II: InjectedIntr,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator
            .text("<intrisic `")
            .append(self.0.name())
            .append("`>")
    }
}

#[cfg(feature = "serde")]
mod serde {

    use serde::{Deserialize, Serialize};

    use super::ValueIntrisic;
    use crate::intrisics::{InjectedIntr, Intrisic};

    #[derive(Deserialize)]
    #[serde(bound = "II: crate::intrisics::InjectedIntr", tag = "$type")]
    enum Serialized<II> {
        #[serde(rename = "intrisic")]
        Nested {
            #[serde(rename = "$intrisic")]
            intrisic: Intrisic<II>,
        },
    }

    #[derive(Serialize)]
    #[serde(bound = "II: crate::intrisics::InjectedIntr", tag = "$type")]
    enum BorrowedSerialized<'m, II> {
        #[serde(rename = "intrisic")]
        Nested {
            #[serde(rename = "$intrisic")]
            intrisic: &'m Intrisic<II>,
        },
    }

    impl<II> Serialize for ValueIntrisic<II>
    where
        II: InjectedIntr,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            BorrowedSerialized::Nested { intrisic: &self.0 }.serialize(serializer)
        }
    }
    impl<'de, II> Deserialize<'de> for ValueIntrisic<II>
    where
        II: InjectedIntr,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let Serialized::Nested { intrisic: name } = Deserialize::deserialize(deserializer)?;
            Ok(Self(name))
        }
    }
}
