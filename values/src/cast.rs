//! Conversions
//!
//! Implement all the possible conversions between concrete types. Injected
//! types are converted if readable, errors otherwise.

use std::{collections::BTreeMap, error::Error};

use num::Zero;
use num::traits::ConstOne;
use num::traits::ConstZero;
use snafu::Snafu;

use crate::{
    Type, Value,
    bool::ValueBool,
    injected::{ReadError, ValueInjected},
    int::ValueInt,
    list::ValueList,
    map::ValueMap,
    null::ValueNull,
    string::ValueString,
};

#[derive(Debug, Snafu)]
pub enum CastError {
    #[snafu(transparent)]
    CastIntoNull { source: CastIntoNullError },
    #[snafu(transparent)]
    CastInjected { source: CastInjectedError },
    #[snafu(transparent)]
    CastIntoInt { source: CastIntoIntError },
    #[snafu(transparent)]
    CastIntoMap { source: CastIntoMapError },
    #[snafu(transparent)]
    UnsupportedCast { source: UnsupportedCast },
}

/// Try to cast `value` into one of the `to` types.
///
/// Casts are tried in order. Return the first successful in [`Ok`], or the
/// errors in [`Err`]
pub fn fall_through_cast(value: Value, to: &[Type]) -> Result<Value, Vec<CastError>> {
    let mut errs = Vec::new();
    for to in to {
        match match to {
            Type::Null => ValueNull::try_from(value.clone())
                .map(Value::from)
                .map_err(CastError::from),
            Type::Bool => ValueBool::try_from(value.clone())
                .map(Value::from)
                .map_err(CastError::from),
            Type::Int => ValueInt::try_from(value.clone())
                .map(Value::from)
                .map_err(CastError::from),
            Type::String => ValueString::try_from(value.clone())
                .map(Value::from)
                .map_err(CastError::from),
            Type::List => ValueList::try_from(value.clone())
                .map(Value::from)
                .map_err(CastError::from),
            Type::Map => ValueMap::try_from(value.clone())
                .map(Value::from)
                .map_err(CastError::from),
            Type::Injected => ValueInjected::try_from(value.clone())
                .map(Value::from)
                .map_err(CastError::from),
        } {
            Ok(value) => return Ok(value),
            Err(err) => errs.push(err),
        }
    }
    Err(errs)
}

/// Remove the possible injected value
///
/// This won't ever return `Ok(Value::Injected(_))`.
pub fn push_down_if_injected(value: Value) -> Result<Value, CastInjectedError> {
    if let Value::Injected(value) = value {
        push_down_injected(value)
    } else {
        Ok(value)
    }
}

/// Read an injected value
///
/// This won't ever return `Ok(Value::Injected(_))`.
pub fn push_down_injected(value: ValueInjected) -> Result<Value, CastInjectedError> {
    value.read().map_err(|err| match err {
        ReadError::NotReadable => CastInjectedError::NotReadable { value },
        ReadError::ReadFailed { source } => CastInjectedError::Read { source },
    })
}

// Some common machinery

#[derive(Debug, Snafu)]
pub enum CastInjectedError {
    #[snafu(display("Value {} is not readable", value.description()))]
    NotReadable { value: ValueInjected },
    #[snafu(transparent)]
    Read { source: Box<dyn Error> },
}

#[derive(Debug, Snafu)]
#[snafu(display("Cannot cast from {from} to {to}"))]
pub struct UnsupportedCast {
    pub from: Type,
    pub to: Type,
}

macro_rules! from_injected {
    ($ty:ty) => {
        from_injected!($ty, err = CastInjectedError);
    };
    ($ty:ty, err=$err:ty) => {
        impl TryFrom<ValueInjected> for $ty {
            type Error = $err;

            fn try_from(value: ValueInjected) -> Result<Self, Self::Error> {
                let value = read_injected(value)?;
                let value = value.try_into()?;
                Ok(value)
            }
        }
    };
}

// ValueNull
//
// All casts except the identity and read injected are unsupported

#[derive(Debug, Snafu)]
pub enum CastIntoNullError {
    #[snafu(transparent)]
    UnsupportedCast { source: UnsupportedCast },
    #[snafu(transparent)]
    CastInjected { source: CastInjectedError },
}

impl TryFrom<ValueBool> for ValueNull {
    type Error = UnsupportedCast;

    fn try_from(_: ValueBool) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Bool,
            to: Type::Null,
        })
    }
}

impl TryFrom<ValueInt> for ValueNull {
    type Error = UnsupportedCast;

    fn try_from(_: ValueInt) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Int,
            to: Type::Null,
        })
    }
}

impl TryFrom<ValueString> for ValueNull {
    type Error = UnsupportedCast;

    fn try_from(_: ValueString) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::String,
            to: Type::Null,
        })
    }
}

impl TryFrom<ValueList> for ValueNull {
    type Error = UnsupportedCast;

    fn try_from(_: ValueList) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::List,
            to: Type::Null,
        })
    }
}

impl TryFrom<ValueMap> for ValueNull {
    type Error = UnsupportedCast;

    fn try_from(_: ValueMap) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Map,
            to: Type::Null,
        })
    }
}

from_injected!(ValueNull, err = CastIntoNullError);

impl TryFrom<Value> for ValueNull {
    type Error = CastIntoNullError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Null(value) => value,
            Value::Bool(value) => value.try_into()?,
            Value::Int(value) => value.try_into()?,
            Value::String(value) => value.try_into()?,
            Value::List(value) => value.try_into()?,
            Value::Map(value) => value.try_into()?,
            Value::Injected(value) => value.try_into()?,
        })
    }
}

// ValueBool
//
// Js inspired truthy operator

impl From<ValueNull> for ValueBool {
    fn from(_: ValueNull) -> Self {
        ValueBool::FALSE
    }
}

impl From<ValueInt> for ValueBool {
    fn from(value: ValueInt) -> Self {
        ValueBool::from(!value.is_zero())
    }
}

impl From<ValueString> for ValueBool {
    fn from(value: ValueString) -> Self {
        ValueBool::from(!value.is_empty())
    }
}

impl From<ValueList> for ValueBool {
    fn from(value: ValueList) -> Self {
        ValueBool::from(!value.is_empty())
    }
}

impl From<ValueMap> for ValueBool {
    fn from(value: ValueMap) -> Self {
        ValueBool::from(!value.is_empty())
    }
}

from_injected!(ValueBool);

impl TryFrom<Value> for ValueBool {
    type Error = CastInjectedError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Null(value) => value.into(),
            Value::Bool(value) => value,
            Value::Int(value) => value.into(),
            Value::String(value) => value.into(),
            Value::List(value) => value.into(),
            Value::Map(value) => value.into(),
            Value::Injected(value) => value.try_into()?,
        })
    }
}

// ValueInt
//
// Cast from bool to 0-1, null to 0. Other are unsupported

#[derive(Debug, Snafu)]
pub enum CastIntoIntError {
    #[snafu(transparent)]
    UnsupportedCast { source: UnsupportedCast },
    #[snafu(transparent)]
    CastInjected { source: CastInjectedError },
}

impl From<ValueNull> for ValueInt {
    fn from(_: ValueNull) -> Self {
        ValueInt::ZERO
    }
}

impl From<ValueBool> for ValueInt {
    fn from(value: ValueBool) -> Self {
        match value.get() {
            true => ValueInt::ONE,
            false => ValueInt::ZERO,
        }
    }
}

impl TryFrom<ValueString> for ValueInt {
    type Error = UnsupportedCast;

    fn try_from(_: ValueString) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::String,
            to: Type::Int,
        })
    }
}

impl TryFrom<ValueList> for ValueInt {
    type Error = UnsupportedCast;

    fn try_from(_: ValueList) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::String,
            to: Type::Int,
        })
    }
}

impl TryFrom<ValueMap> for ValueInt {
    type Error = UnsupportedCast;

    fn try_from(_: ValueMap) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::String,
            to: Type::Int,
        })
    }
}

from_injected!(ValueInt, err = CastIntoIntError);

impl TryFrom<Value> for ValueInt {
    type Error = CastIntoIntError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Null(value) => value.into(),
            Value::Bool(value) => value.into(),
            Value::Int(value) => value,
            Value::String(value) => value.try_into()?,
            Value::List(value) => value.try_into()?,
            Value::Map(value) => value.try_into()?,
            Value::Injected(value) => value.try_into()?,
        })
    }
}

// ValueString
//
// Except the identity ValueString -> ValueString, the other are printed using `Display`

impl From<ValueNull> for ValueString {
    fn from(_: ValueNull) -> Self {
        ValueString::new_static("null")
    }
}

impl From<ValueBool> for ValueString {
    fn from(value: ValueBool) -> Self {
        match value.get() {
            true => ValueString::new_static("true"),
            false => ValueString::new_static("false"),
        }
    }
}

impl From<ValueInt> for ValueString {
    fn from(value: ValueInt) -> Self {
        ValueString::new(value.to_string())
    }
}

impl From<ValueList> for ValueString {
    fn from(value: ValueList) -> Self {
        ValueString::new(value.to_string())
    }
}

impl From<ValueMap> for ValueString {
    fn from(value: ValueMap) -> Self {
        ValueString::new(value.to_string())
    }
}

from_injected!(ValueString);

impl TryFrom<Value> for ValueString {
    type Error = CastInjectedError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Null(value) => value.into(),
            Value::Bool(value) => value.into(),
            Value::Int(value) => value.into(),
            Value::String(value) => value,
            Value::List(value) => value.into(),
            Value::Map(value) => value.into(),
            Value::Injected(value) => value.try_into()?,
        })
    }
}

// ValueList
//
// nulls, bools and ints goes to the list with only an element. Strings becomes
// lists of chars and maps lists of key-values pairs.

impl From<ValueNull> for ValueList {
    fn from(value: ValueNull) -> Self {
        ValueList::new(vec![Value::Null(value)])
    }
}

impl From<ValueBool> for ValueList {
    fn from(value: ValueBool) -> Self {
        ValueList::new(vec![Value::Bool(value)])
    }
}

impl From<ValueInt> for ValueList {
    fn from(value: ValueInt) -> Self {
        ValueList::new(vec![Value::Int(value)])
    }
}

impl From<ValueString> for ValueList {
    fn from(value: ValueString) -> Self {
        let mut values = Vec::with_capacity(value.len());
        for (pos, ch) in value.char_indices() {
            values.push(Value::String(
                value.slice(pos..(pos + ch.len_utf8())).unwrap(),
            ));
        }
        ValueList::new(values)
    }
}

impl From<ValueMap> for ValueList {
    fn from(value: ValueMap) -> Self {
        ValueList::from_iter(
            value
                .into_iter()
                .map(|(k, v)| Value::List(ValueList::new(vec![Value::String(k), v]))),
        )
    }
}

from_injected!(ValueList);

impl TryFrom<Value> for ValueList {
    type Error = CastInjectedError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Null(value) => value.into(),
            Value::Bool(value) => value.into(),
            Value::Int(value) => value.into(),
            Value::String(value) => value.into(),
            Value::List(value) => value,
            Value::Map(value) => value.into(),
            Value::Injected(value) => value.try_into()?,
        })
    }
}

// ValueMap
//
// null, bools, ints and strings are unsupported. Lists of tuples roundtrip,
// otherwise lists throw a descriptive error

#[derive(Debug, Snafu)]
pub enum CastIntoMapError {
    #[snafu(transparent)]
    UnsupportedCast { source: UnsupportedCast },
    #[snafu(transparent)]
    CastInjected { source: CastInjectedError },
    #[snafu(display("List entry is not a list, but a {ty}"))]
    EntryNotAList { ty: Type },
    #[snafu(display("List entry is not a pair, has length {len}"))]
    EntryNotAPair { len: usize },
    #[snafu(display("Map key must be a string, got {ty}"))]
    KeyNotString { ty: Type },
}

impl TryFrom<ValueNull> for ValueMap {
    type Error = UnsupportedCast;

    fn try_from(_: ValueNull) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Null,
            to: Type::Map,
        })
    }
}

impl TryFrom<ValueBool> for ValueMap {
    type Error = UnsupportedCast;

    fn try_from(_: ValueBool) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Bool,
            to: Type::Map,
        })
    }
}

impl TryFrom<ValueInt> for ValueMap {
    type Error = UnsupportedCast;

    fn try_from(_: ValueInt) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Int,
            to: Type::Map,
        })
    }
}

impl TryFrom<ValueString> for ValueMap {
    type Error = UnsupportedCast;

    fn try_from(_: ValueString) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::String,
            to: Type::Map,
        })
    }
}

impl TryFrom<ValueList> for ValueMap {
    type Error = CastIntoMapError;

    fn try_from(value: ValueList) -> Result<Self, Self::Error> {
        let mut map = BTreeMap::new();
        for entry in value {
            let entry_list: ValueList = entry
                .try_unwrap_list()
                .map_err(|v| CastIntoMapError::EntryNotAList { ty: v.input.typ() })?;
            let mut iter = entry_list.into_iter();
            if iter.len() != 2 {
                return Err(CastIntoMapError::EntryNotAPair { len: iter.len() });
            }
            let key = iter.next().unwrap();
            let val = iter.next().unwrap();
            let key = key
                .try_unwrap_string()
                .map_err(|v| CastIntoMapError::KeyNotString { ty: v.input.typ() })?;
            map.insert(key, val);
        }
        Ok(ValueMap::new(map))
    }
}

from_injected!(ValueMap, err = CastIntoMapError);

impl TryFrom<Value> for ValueMap {
    type Error = CastIntoMapError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Null(value) => value.try_into()?,
            Value::Bool(value) => value.try_into()?,
            Value::Int(value) => value.try_into()?,
            Value::String(value) => value.try_into()?,
            Value::List(value) => value.try_into()?,
            Value::Map(value) => value,
            Value::Injected(value) => value.try_into()?,
        })
    }
}

// ValueInjected
//
// All casts except the identity are unsupported

impl TryFrom<ValueNull> for ValueInjected {
    type Error = UnsupportedCast;

    fn try_from(_: ValueNull) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Null,
            to: Type::Injected,
        })
    }
}

impl TryFrom<ValueBool> for ValueInjected {
    type Error = UnsupportedCast;

    fn try_from(_: ValueBool) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Bool,
            to: Type::Injected,
        })
    }
}

impl TryFrom<ValueInt> for ValueInjected {
    type Error = UnsupportedCast;

    fn try_from(_: ValueInt) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Int,
            to: Type::Injected,
        })
    }
}

impl TryFrom<ValueString> for ValueInjected {
    type Error = UnsupportedCast;

    fn try_from(_: ValueString) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::String,
            to: Type::Injected,
        })
    }
}

impl TryFrom<ValueList> for ValueInjected {
    type Error = UnsupportedCast;

    fn try_from(_: ValueList) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::List,
            to: Type::Injected,
        })
    }
}

impl TryFrom<ValueMap> for ValueInjected {
    type Error = UnsupportedCast;

    fn try_from(_: ValueMap) -> Result<Self, Self::Error> {
        Err(UnsupportedCast {
            from: Type::Map,
            to: Type::Injected,
        })
    }
}

impl TryFrom<Value> for ValueInjected {
    type Error = UnsupportedCast;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        Ok(match value {
            Value::Null(value) => value.try_into()?,
            Value::Bool(value) => value.try_into()?,
            Value::Int(value) => value.try_into()?,
            Value::String(value) => value.try_into()?,
            Value::List(value) => value.try_into()?,
            Value::Map(value) => value.try_into()?,
            Value::Injected(value) => value,
        })
    }
}
