//! Serialize Rust types into a dices [`Value`].
//!
//! The [`serde`] data model is mapped onto the dices value model as follows:
//!
//! - booleans, integers, strings map to their obvious value kinds;
//! - `char` becomes a one-character string;
//! - `unit`, unit structs and `None` become [`Value::Null`];
//! - `Some(v)` and newtype structs are transparent (serialize their content);
//! - sequences, tuples and tuple structs become a [`Value::List`];
//! - maps and structs become a [`Value::Map`];
//! - enums use the externally-tagged convention: a unit variant is the bare
//!   variant name as a string, every other variant is a one-entry map from the
//!   variant name to its content.
//!
//! Floating point numbers have no representation in the value model and produce
//! [`Error::FloatUnsupported`].

use std::collections::BTreeMap;

use num::{NumCast, ToPrimitive};
use serde::{
    Serialize, Serializer,
    ser::{self, SerializeMap as _, SerializeSeq as _},
};

use super::error::{Error, Result};
use crate::{
    Value, bool::ValueBool, int::ValueInt, list::ValueList, map::ValueMap, null::ValueNull,
    string::ValueString,
};

/// Serialize a value into a [`Value`].
pub fn to_value<T: Serialize>(value: &T) -> Result<Value> {
    value.serialize(ValueSerializer)
}

/// Serialize a value implementing [`Serialize`] into a dices [`Value`].
#[derive(Debug, Clone, Copy)]
pub struct ValueSerializer;

/// Build a [`ValueInt`] from any primitive integer.
///
/// Every primitive integer fits in a [`ValueInt`] (which is unbounded), so the
/// cast can never fail.
fn value_int<T: num::ToPrimitive + Copy + std::fmt::Debug>(n: T) -> ValueInt {
    NumCast::from(n).expect("every primitive integer fits in an unbounded ValueInt")
}

/// Wrap a value as a single-entry map keyed by an enum variant name.
fn singleton_map(variant: &'static str, value: Value) -> Value {
    let mut entries = BTreeMap::new();
    entries.insert(ValueString::new_static(variant), value);
    Value::Map(ValueMap::new(entries))
}

impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;
    type SerializeSeq = SerializeSeq;
    type SerializeTuple = SerializeSeq;
    type SerializeTupleStruct = SerializeSeq;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeStruct;
    type SerializeStructVariant = SerializeStructVariant;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(ValueBool::from(v)))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Int(value_int(v)))
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::FloatUnsupported)
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::FloatUnsupported)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(ValueString::new(v.to_string())))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(ValueString::new(v.to_owned())))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let items = v.iter().map(|b| Value::Int(value_int(*b))).collect();
        Ok(Value::List(ValueList::new(items)))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null(ValueNull))
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null(ValueNull))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Null(ValueNull))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(ValueString::new_static(variant)))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Ok(singleton_map(variant, value.serialize(ValueSerializer)?))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeSeq {
            items: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeSeq {
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeSeq {
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeTupleVariant {
            variant,
            items: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap {
            entries: BTreeMap::new(),
            next_key: None,
        })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeStruct {
            entries: BTreeMap::new(),
        })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeStructVariant {
            variant,
            entries: BTreeMap::new(),
        })
    }
}

/// Collects sequence, tuple and tuple-struct elements into a [`Value::List`].
pub struct SerializeSeq {
    items: Vec<Value>,
}

impl ser::SerializeSeq for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(ValueList::new(self.items)))
    }
}

impl ser::SerializeTuple for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(ValueList::new(self.items)))
    }
}

impl ser::SerializeTupleStruct for SerializeSeq {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(ValueList::new(self.items)))
    }
}

/// Collects a tuple-variant's fields into a `{ variant: [..] }` map.
pub struct SerializeTupleVariant {
    variant: &'static str,
    items: Vec<Value>,
}

impl ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.items.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(singleton_map(
            self.variant,
            Value::List(ValueList::new(self.items)),
        ))
    }
}

/// Collects map entries into a [`Value::Map`], coercing keys to strings.
pub struct SerializeMap {
    entries: BTreeMap<ValueString, Value>,
    next_key: Option<ValueString>,
}

impl ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = self
            .next_key
            .take()
            .expect("serialize_value called before serialize_key");
        self.entries.insert(key, value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Map(ValueMap::new(self.entries)))
    }
}

/// Collects struct fields into a [`Value::Map`].
pub struct SerializeStruct {
    entries: BTreeMap<ValueString, Value>,
}

impl ser::SerializeStruct for SerializeStruct {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.entries.insert(
            ValueString::new_static(key),
            value.serialize(ValueSerializer)?,
        );
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Map(ValueMap::new(self.entries)))
    }
}

/// Collects a struct-variant's fields into a `{ variant: { .. } }` map.
pub struct SerializeStructVariant {
    variant: &'static str,
    entries: BTreeMap<ValueString, Value>,
}

impl ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.entries.insert(
            ValueString::new_static(key),
            value.serialize(ValueSerializer)?,
        );
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(singleton_map(
            self.variant,
            Value::Map(ValueMap::new(self.entries)),
        ))
    }
}

/// Serializer used for map keys, which must reduce to a dices string.
///
/// Strings are taken verbatim; integers, booleans and chars are stringified so
/// that common map key types map seamlessly. Anything else is rejected with
/// [`Error::InvalidMapKey`].
struct MapKeySerializer;

/// Helper for the many `MapKeySerializer` methods that cannot produce a key.
type KeyImpossible = ser::Impossible<ValueString, Error>;

impl MapKeySerializer {
    fn from_display(value: impl std::fmt::Display) -> Result<ValueString, Error> {
        Ok(ValueString::new(value.to_string()))
    }
}

impl Serializer for MapKeySerializer {
    type Ok = ValueString;
    type Error = Error;
    type SerializeSeq = KeyImpossible;
    type SerializeTuple = KeyImpossible;
    type SerializeTupleStruct = KeyImpossible;
    type SerializeTupleVariant = KeyImpossible;
    type SerializeMap = KeyImpossible;
    type SerializeStruct = KeyImpossible;
    type SerializeStructVariant = KeyImpossible;

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(ValueString::new(v.to_owned()))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Self::from_display(v)
    }

    fn serialize_f32(self, _v: f32) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKey { found: "float" })
    }

    fn serialize_f64(self, _v: f64) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKey { found: "float" })
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKey { found: "bytes" })
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKey { found: "null" })
    }

    fn serialize_some<T>(self, _value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::InvalidMapKey { found: "option" })
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKey { found: "null" })
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(Error::InvalidMapKey { found: "null" })
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(ValueString::new_static(variant))
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        Err(Error::InvalidMapKey { found: "enum" })
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(Error::InvalidMapKey { found: "list" })
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(Error::InvalidMapKey { found: "list" })
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(Error::InvalidMapKey { found: "list" })
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(Error::InvalidMapKey { found: "enum" })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Err(Error::InvalidMapKey { found: "map" })
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Err(Error::InvalidMapKey { found: "map" })
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(Error::InvalidMapKey { found: "enum" })
    }
}

/// Serialize a [`Value`] onto any [`Serializer`].
///
/// This is the mirror of [`ValueDeserializer::deserialize_any`], so a value
/// fed through [`ValueSerializer`] reproduces itself exactly. The integer probe
/// order (`i64` → `u64` → `i128` → `u128`) matches the deserializer.
///
/// Integers larger than `u128` have no representation in the `serde` data model
/// and produce an error, mirroring [`ValueDeserializer`], which already rejects
/// them on the way out.
///
/// [`ValueDeserializer`]: crate::serde::de::ValueDeserializer
/// [`ValueDeserializer::deserialize_any`]: crate::serde::de::ValueDeserializer
impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Value::Null(_) => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(b.get()),
            Value::Int(n) => {
                if let Some(v) = n.to_i64() {
                    serializer.serialize_i64(v)
                } else if let Some(v) = n.to_u64() {
                    serializer.serialize_u64(v)
                } else if let Some(v) = n.to_i128() {
                    serializer.serialize_i128(v)
                } else if let Some(v) = n.to_u128() {
                    serializer.serialize_u128(v)
                } else {
                    Err(ser::Error::custom(format!(
                        "integer {n} is too large to serialize (exceeds u128)"
                    )))
                }
            }
            Value::String(s) => serializer.serialize_str(s.as_str()),
            Value::List(list) => {
                let mut seq = serializer.serialize_seq(Some(list.len()))?;
                for item in list.iter() {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            Value::Map(map) => {
                let mut entries = serializer.serialize_map(Some(map.len()))?;
                for (key, value) in map.iter() {
                    entries.serialize_entry(key.as_str(), value)?;
                }
                entries.end()
            }
        }
    }
}
