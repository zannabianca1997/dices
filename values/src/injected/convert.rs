//! Support machinery for the `#[injectable]` attribute macro.
//!
//! Mostly an application of the autoref specialization to enable using both
//! directly convertible types and Serialize/Deserialize implementations
//!
//! See <https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md>.

use std::{error::Error, marker::PhantomData};

use serde::{Serialize, de::DeserializeOwned};
use snafu::Snafu;

use crate::Value;

type BoxError = Box<dyn Error>;

/// Error raised when an injectable is called with the wrong number of arguments.
#[derive(Debug, Snafu)]
#[snafu(display("expected {expected} argument(s), got {got}"))]
pub struct WrongArgCount {
    pub expected: usize,
    pub got: usize,
}

/// Tag carrying the target type of an argument conversion.
pub struct ArgTag<T>(PhantomData<T>);

impl<T> ArgTag<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for ArgTag<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback argument conversion via [`serde`] (lower method-resolution priority).
pub trait ViaDeserialize<T> {
    fn convert(&self, value: Value) -> Result<T, BoxError>;
}

impl<T> ViaDeserialize<T> for ArgTag<T>
where
    T: DeserializeOwned,
{
    fn convert(&self, value: Value) -> Result<T, BoxError> {
        crate::serde::from_value(value).map_err(|err| Box::new(err) as BoxError)
    }
}

/// Preferred argument conversion via [`TryFrom`] (higher method-resolution priority).
pub trait ViaTryFrom<T> {
    fn convert(&self, value: Value) -> Result<T, BoxError>;
}

impl<T> ViaTryFrom<T> for &ArgTag<T>
where
    T: TryFrom<Value>,
    T::Error: Into<BoxError>,
{
    fn convert(&self, value: Value) -> Result<T, BoxError> {
        T::try_from(value).map_err(Into::into)
    }
}

/// Tag carrying the source type of a return-value conversion.
pub struct RetTag<T>(PhantomData<T>);

impl<T> RetTag<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Default for RetTag<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Fallback return conversion via [`serde`] (lower method-resolution priority).
pub trait ReturnViaSerialize<T> {
    fn convert(&self, value: T) -> Result<Value, BoxError>;
}

impl<T> ReturnViaSerialize<T> for RetTag<T>
where
    T: Serialize,
{
    fn convert(&self, value: T) -> Result<Value, BoxError> {
        crate::serde::to_value(&value).map_err(|err| Box::new(err) as BoxError)
    }
}

/// Preferred return conversion via [`TryInto`] (higher method-resolution priority).
pub trait ReturnViaTryInto<T> {
    fn convert(&self, value: T) -> Result<Value, BoxError>;
}

impl<T> ReturnViaTryInto<T> for &RetTag<T>
where
    T: TryInto<Value>,
    T::Error: Into<BoxError>,
{
    fn convert(&self, value: T) -> Result<Value, BoxError> {
        value.try_into().map_err(Into::into)
    }
}

/// Fallback return conversion via [`serde`] (lower method-resolution priority).
pub trait ReturnFallibleViaSerialize<T> {
    fn convert(&self, value: T) -> Result<Value, BoxError>;
}

impl<T, E> ReturnFallibleViaSerialize<Result<T, E>> for &&RetTag<Result<T, E>>
where
    T: Serialize,
    E: Error + 'static,
{
    fn convert(&self, value: Result<T, E>) -> Result<Value, BoxError> {
        let value = value.map_err(|err| Box::new(err) as BoxError)?;
        crate::serde::to_value(&value).map_err(|err| Box::new(err) as BoxError)
    }
}

/// Preferred return conversion via [`TryInto`] (higher method-resolution priority).
pub trait ReturnFallibleViaTryInto<T> {
    fn convert(&self, value: T) -> Result<Value, BoxError>;
}

impl<T, E> ReturnFallibleViaTryInto<Result<T, E>> for &&&RetTag<Result<T, E>>
where
    T: TryInto<Value>,
    T::Error: Into<BoxError>,
    E: Error + 'static,
{
    fn convert(&self, value: Result<T, E>) -> Result<Value, BoxError> {
        let value = value.map_err(|err| Box::new(err) as BoxError)?;
        value.try_into().map_err(Into::into)
    }
}
