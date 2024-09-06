//! Value enclosing an expression

use std::{collections::BTreeMap, fmt::Display};

use pretty::{DocAllocator, Pretty};

use crate::{
    expression::Expression, ident::IdentStr, intrisics::InjectedIntr, values::number::ValueNumber,
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

impl<'a, D, A, II> Pretty<'a, D, A> for &'a ValueClosure<II>
where
    A: 'a,
    D: ?Sized + DocAllocator<'a, A>,
    II: InjectedIntr,
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
