//! Value enclosing an expression

use std::{collections::BTreeMap, fmt::Display};

use crate::{
    expression::Expression, ident::IdentStr, intrisics::NoInjectedIntrisics,
    value::number::ValueNumber,
};

use super::{list::ValueList, ToNumberError, Value};

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
#[cfg_attr(
    feature = "bincode",
    derive(bincode::Decode, bincode::Encode,),
    bincode(bounds = "InjectedIntrisic: crate::intrisics::InjectedIntr")
)]
pub struct ValueClosure<InjectedIntrisic> {
    pub params: Box<[Box<IdentStr>]>,
    pub captures: BTreeMap<Box<IdentStr>, Value<InjectedIntrisic>>,
    pub body: Expression<InjectedIntrisic>,
}
impl<InjectedIntrisic> ValueClosure<InjectedIntrisic> {
    pub fn to_number(self) -> Result<ValueNumber, crate::value::ToNumberError> {
        Err(ToNumberError::Closure)
    }
    pub fn to_list(self) -> Result<ValueList<InjectedIntrisic>, super::ToListError> {
        Ok(ValueList::from_iter([Box::new(self).into()]))
    }
}

impl ValueClosure<NoInjectedIntrisics> {
    // Add any intrisic type to a intrisic-less value
    pub fn with_arbitrary_injected_intrisics<II>(self) -> ValueClosure<II> {
        let ValueClosure {
            params,
            captures,
            body,
        } = self;
        ValueClosure {
            params,
            captures: captures
                .into_iter()
                .map(|(k, v)| (k, v.with_arbitrary_injected_intrisics()))
                .collect(),
            body: body.with_arbitrary_injected_intrisics(),
        }
    }
}

impl<InjectedIntrisic> Display for ValueClosure<InjectedIntrisic> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<closure")?;
        if self.params.is_empty() {
            write!(f, " without parameters")?
        } else {
            write!(f, " with {} parameters", self.params.len())?
        };
        if !self.captures.is_empty() {
            write!(f, " (captured {} values)", self.captures.len())?
        };
        write!(f, ">")?;
        Ok(())
    }
}

#[cfg(feature = "pretty")]
impl<'a, D, A, II> pretty::Pretty<'a, D, A> for &'a ValueClosure<II>
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
    II: crate::intrisics::InjectedIntr,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        let text = allocator.text("<closure");
        let text = if self.params.is_empty() {
            text.append(" without parameters")
        } else {
            text.append(" with ")
                .append(self.params.len().to_string())
                .append(" parameters")
        };
        let text = if !self.captures.is_empty() {
            text.append(" (captured ")
                .append(self.captures.len().to_string())
                .append(" values)")
        } else {
            text
        };
        text.append(">")
    }
}

#[cfg(feature = "serde")]
mod serde {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};
    use serde_bytes::ByteBuf;

    use super::ValueClosure;
    use crate::{ident::IdentStr, intrisics::InjectedIntr, Value};

    #[derive(Deserialize)]
    #[serde(bound = "InjectedIntrisic: InjectedIntr", tag = "$type")]
    enum Serialized<InjectedIntrisic> {
        #[serde(rename = "closure")]
        Nested {
            #[serde(rename = "$params")]
            params: Box<[Box<IdentStr>]>,
            #[serde(rename = "$captures", default)]
            captures: BTreeMap<Box<IdentStr>, Value<InjectedIntrisic>>,
            #[serde(rename = "$body")]
            body: ByteBuf,
        },
    }

    #[derive(Serialize)]
    #[serde(bound = "InjectedIntrisic: InjectedIntr", tag = "$type")]
    enum BorrowedSerialized<'m, InjectedIntrisic> {
        #[serde(rename = "closure")]
        Nested {
            #[serde(rename = "$params")]
            params: &'m [Box<IdentStr>],
            #[serde(rename = "$captures", skip_serializing_if = "BTreeMap::is_empty")]
            captures: &'m BTreeMap<Box<IdentStr>, Value<InjectedIntrisic>>,
            #[serde(rename = "$body")]
            body: ByteBuf,
        },
    }

    impl<II> Serialize for ValueClosure<II>
    where
        II: InjectedIntr,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            BorrowedSerialized::Nested {
                params: &self.params,
                captures: &self.captures,
                body: ByteBuf::from(
                    bincode::encode_to_vec(&self.body, bincode::config::standard())
                        .map_err(<S::Error as serde::ser::Error>::custom)?,
                ),
            }
            .serialize(serializer)
        }
    }
    impl<'de, II> Deserialize<'de> for ValueClosure<II>
    where
        II: InjectedIntr,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let Serialized::Nested {
                params,
                captures,
                body,
            } = Deserialize::deserialize(deserializer)?;
            Ok(Self {
                params,
                captures,
                body: bincode::decode_from_slice(&body, bincode::config::standard())
                    .map_err(<D::Error as serde::de::Error>::custom)?
                    .0,
            })
        }
    }
}
