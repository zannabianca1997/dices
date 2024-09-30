use std::{collections::BTreeMap, fmt::Display};

use itertools::Itertools;

use crate::{fmt::quoted_if_not_ident, intrisics::InjectedIntr};

use super::{list::ValueList, string::ValueString, Value};

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
    bincode(bounds = "InjectedIntrisic: InjectedIntr + 'static")
)]
pub struct ValueMap<InjectedIntrisic>(BTreeMap<ValueString, Value<InjectedIntrisic>>);
impl<InjectedIntrisic> ValueMap<InjectedIntrisic> {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    #[cfg(feature = "parse_value")]
    pub fn to_number(self) -> Result<super::number::ValueNumber, super::ToNumberError> {
        match self.0.into_iter().exactly_one() {
            Ok((_, value)) => value.to_number(),
            Err(vals) => Err(super::ToNumberError::WrongListLength(vals.len())),
        }
    }

    pub fn to_list(self) -> Result<ValueList<InjectedIntrisic>, super::ToListError> {
        Ok(self.0.into_values().collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&ValueString, &Value<InjectedIntrisic>)> {
        self.0.iter()
    }
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (&ValueString, &mut Value<InjectedIntrisic>)> {
        self.0.iter_mut()
    }

    pub fn get(&self, key: &str) -> Option<&Value<InjectedIntrisic>> {
        self.0.get(key)
    }
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value<InjectedIntrisic>> {
        self.0.get_mut(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value<InjectedIntrisic>> {
        self.0.remove(key)
    }

    pub fn insert(
        &mut self,
        key: ValueString,
        value: Value<InjectedIntrisic>,
    ) -> Option<Value<InjectedIntrisic>> {
        self.0.insert(key, value)
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }
}

impl<II: InjectedIntr> Display for ValueMap<II> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        struct KeyValue<'m, II>((&'m ValueString, &'m super::Value<II>));
        impl<II: InjectedIntr> Display for KeyValue<'_, II> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let (idx, val) = self.0;
                quoted_if_not_ident(&idx, f)?;
                write!(f, ": {val}")
            }
        }

        write!(f, "<|{}|>", self.0.iter().map(KeyValue).format(", "))
    }
}

#[cfg(feature = "pretty")]
impl<'a, D, A, II> pretty::Pretty<'a, D, A> for &'a ValueMap<II>
where
    A: 'a,
    D: ?Sized + pretty::DocAllocator<'a, A>,
    II: InjectedIntr,
{
    fn pretty(self, allocator: &'a D) -> pretty::DocBuilder<'a, D, A> {
        allocator
            .intersperse(
                self.iter().map(|(key, value)| {
                    struct QuoteIfNotIdent<'a>(&'a str);
                    impl Display for QuoteIfNotIdent<'_> {
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            quoted_if_not_ident(self.0, f)
                        }
                    }
                    allocator
                        .text(QuoteIfNotIdent(&key).to_string())
                        .append(":")
                        .append(allocator.space())
                        .append(value)
                }),
                crate::fmt::CommaLine,
            )
            .enclose(allocator.line_(), allocator.line_())
            .group()
            .nest(4)
            .enclose("<|", "|>")
    }
}

impl<InjectedIntrisic> FromIterator<(ValueString, Value<InjectedIntrisic>)>
    for ValueMap<InjectedIntrisic>
{
    fn from_iter<T: IntoIterator<Item = (ValueString, Value<InjectedIntrisic>)>>(iter: T) -> Self {
        Self(FromIterator::from_iter(iter))
    }
}
impl<InjectedIntrisic> IntoIterator for ValueMap<InjectedIntrisic> {
    type Item = (ValueString, Value<InjectedIntrisic>);

    type IntoIter = <BTreeMap<ValueString, Value<InjectedIntrisic>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
