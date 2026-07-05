use std::{collections::BTreeMap, error::Error};

use derive_more::Unwrap;
use serde::Serialize;

use crate::{Value, injected::Injectable, string::ValueString};

#[derive(Debug, Unwrap)]
pub enum ReadValue<'a> {
    Value(Value),
    Map(BTreeMap<ValueString, ValueOrInject<'a>>),
    List(Vec<ValueOrInject<'a>>),
}

#[derive(Debug)]
pub enum ValueOrInject<'a> {
    Value(ReadValue<'a>),
    Inject(&'a dyn Injectable),
}

/// Wrapped value is readable as a dices value
pub trait Readable {
    fn read(&self) -> Result<ReadValue<'_>, Box<dyn Error>>;
}

/// Implement [`Readable`] on an object using its [`Serialize`] implementation
pub trait ReadableWithSerde: Serialize {}

impl<T> Readable for T
where
    T: ReadableWithSerde,
{
    fn read(&self) -> Result<ReadValue<'_>, Box<dyn Error>> {
        crate::serde::to_value(self)
            .map(ReadValue::Value)
            .map_err(|err| Box::new(err) as Box<dyn Error>)
    }
}
