//! Value enclosing an expression

use std::fmt::Display;

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
pub struct ValueClosure {
    pub params: Box<[Box<IdentStr>]>,
    pub captured: Box<[(Box<IdentStr>, Value)]>,
    pub body: Expression,
}
impl ValueClosure {
    pub fn to_number(self) -> Result<ValueNumber, crate::values::ToNumberError> {
        Err(ToNumberError::Closure)
    }
    pub fn to_list(self) -> Result<ValueList, super::ToListError> {
        Ok(ValueList::from_iter([Box::new(self).into()]))
    }
}

impl Display for ValueClosure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<closure")?;
        if self.params.is_empty() {
            write!(f, " without parameters")?
        } else {
            write!(f, " with {} parameters", self.params.len())?
        };
        if !self.captured.is_empty() {
            write!(f, " (captured {} values)", self.captured.len())?
        };
        write!(f, ">")?;
        Ok(())
    }
}
