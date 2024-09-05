//! Value enclosing an expression

use std::{collections::BTreeMap, fmt::Display};

use crate::{expression::Expression, ident::IdentStr, values::number::ValueNumber};

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
pub struct ValueClosure<InjectedIntrisic> {
    pub params: Box<[Box<IdentStr>]>,
    pub captures: BTreeMap<Box<IdentStr>, Value<InjectedIntrisic>>,
    pub body: Expression<InjectedIntrisic>,
}
impl<InjectedIntrisic> ValueClosure<InjectedIntrisic> {
    pub fn to_number(self) -> Result<ValueNumber, crate::values::ToNumberError> {
        Err(ToNumberError::Closure)
    }
    pub fn to_list(self) -> Result<ValueList<InjectedIntrisic>, super::ToListError> {
        Ok(ValueList::from_iter([Box::new(self).into()]))
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
