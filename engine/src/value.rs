use std::{collections::HashMap, fmt::Display, rc::Rc};

use strum::{EnumDiscriminants, EnumIs, EnumTryAs, IntoStaticStr};
use thiserror::Error;

use crate::expr::Statement;
use crate::identifier::DIdentifier;

/// String type in `dices`
pub type DString = Rc<str>;

#[derive(Debug, Clone, EnumDiscriminants, EnumTryAs, PartialEq, Eq)]
#[strum_discriminants(name(Type), derive(EnumIs, IntoStaticStr))]
#[cfg_attr(
    feature = "serde",
    derive(::serde::Serialize, ::serde::Deserialize),
    serde(tag = "type", content = "value", deny_unknown_fields)
)]
/// Value of a variable in `dices`
pub enum Value {
    /// A callable
    Function {
        /// Parameters of the function
        params: Rc<[DIdentifier]>,
        /// Captured context
        context: HashMap<DIdentifier, Value>,
        /// Body of the function
        body: Rc<[Statement]>,
    },

    // Plain data types
    /// Null
    Null,
    /// Boolean
    Bool(bool),
    /// Number
    Number(i64),
    /// List
    List(Vec<Self>),
    /// String
    String(DString),
    /// Map
    Map(HashMap<DString, Self>),
}

impl Value {
    pub fn to_number(&self) -> Result<i64, ToNumberError> {
        Ok(match self {
            Value::Bool(b) => match b {
                true => 1,
                false => 0,
            },
            Value::Number(n) => *n,
            Value::List(l) => {
                if let [n] = &**l {
                    n.to_number()?
                } else {
                    return Err(ToNumberError::ListNotSingular(l.len()));
                }
            }

            _ => return Err(ToNumberError::InvalidType(self.type_())),
        })
    }

    pub fn type_(&self) -> Type {
        Type::from(self)
    }
}

#[derive(Debug, Error, Clone, Copy)]
pub enum ToNumberError {
    #[error("List of length {0} is not a valid number")]
    ListNotSingular(usize),
    #[error("Type {0} is not acceptable as a number")]
    InvalidType(Type),
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(<&'static str>::from(self), f)
    }
}
