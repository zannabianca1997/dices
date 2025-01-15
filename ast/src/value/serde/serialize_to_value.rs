/**
 * Serializing into a `dices` value
 *
 * A lot of this code is heavily inspired (and in some part straight up copied) from
 * the eccellent work of [dtolnay](https://github.com/dtolnay) in his library `serde_json`
 */
use std::fmt::Display;

use derive_more::derive::From;
use serde::{
    ser::{
        SerializeSeq, SerializeStruct, SerializeStructVariant, SerializeTuple,
        SerializeTupleStruct, SerializeTupleVariant,
    },
    Serialize, Serializer,
};
use thiserror::Error;

use crate::{
    intrisics::NoInjectedIntrisics,
    value::{number::FloatTooBig, ValueMap, ValueNull, ValueString},
    Value,
};

#[derive(Debug, From, Clone, Error)]
pub enum Error {
    #[error("{_0}")]
    #[from(skip)]
    Custom(String),
    #[from(skip)]
    #[error("All keys must be string, not {_0}")]
    NonStringKey(Value),
    #[error("A f32 did not fit in a ValueNumber")]
    F32TooBig(#[source] FloatTooBig<f32>),
    #[error("A f64 did not fit in a ValueNumber")]
    F64TooBig(#[source] FloatTooBig<f64>),
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

struct SerializeList {
    variant_name: Option<&'static str>,
    values: Vec<Value>,
}

impl SerializeList {
    fn new(len: Option<usize>) -> Self {
        Self {
            variant_name: None,
            values: len.map(Vec::with_capacity).unwrap_or_default(),
        }
    }
    fn new_variant(name: &'static str, len: Option<usize>) -> Self {
        Self {
            variant_name: Some(name),
            values: len.map(Vec::with_capacity).unwrap_or_default(),
        }
    }
}

impl SerializeSeq for SerializeList {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.values.into()))
    }
}

impl SerializeTuple for SerializeList {
    type Ok = Value;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.values.into()))
    }
}

impl SerializeTupleStruct for SerializeList {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(self.values.into()))
    }
}

impl SerializeTupleVariant for SerializeList {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let name = self.variant_name.unwrap();
        let mut map = ValueMap::new();
        map.insert(name.into(), Value::List(self.values.into()));
        Ok(map.into())
    }
}

struct SerializeMap {
    variant_name: Option<&'static str>,
    map: ValueMap<NoInjectedIntrisics>,
    key: Option<ValueString>,
}
impl SerializeMap {
    const fn new(_len: Option<usize>) -> Self {
        Self {
            variant_name: None,
            map: ValueMap::new(),
            key: None,
        }
    }
    const fn new_variant(name: &'static str, _len: Option<usize>) -> Self {
        Self {
            variant_name: Some(name),
            map: ValueMap::new(),
            key: None,
        }
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.key = Some(
            key.serialize(ValueSerializer)?
                .into_string()
                .map_err(Error::NonStringKey)?,
        );
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        let key = self.key.take().unwrap();
        self.map.insert(key, value.serialize(ValueSerializer)?);
        Ok(())
    }
    fn serialize_entry<K, V>(&mut self, key: &K, value: &V) -> Result<(), Self::Error>
    where
        K: ?Sized + Serialize,
        V: ?Sized + Serialize,
    {
        self.map.insert(
            key.serialize(ValueSerializer)?
                .into_string()
                .map_err(Error::NonStringKey)?,
            value.serialize(ValueSerializer)?,
        );
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.map.into())
    }
}

impl SerializeStruct for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.map
            .insert(key.into(), value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(self.map.into())
    }
}
impl SerializeStructVariant for SerializeMap {
    type Ok = Value;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize,
    {
        self.map
            .insert(key.into(), value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        let name = self.variant_name.unwrap();
        let mut map = ValueMap::new();
        map.insert(name.into(), self.map.into());
        Ok(map.into())
    }
}

/// Serializer that emits a `dices` value
struct ValueSerializer;

impl Serializer for ValueSerializer {
    type Ok = Value;
    type Error = Error;

    type SerializeSeq = SerializeList;
    type SerializeTuple = SerializeList;
    type SerializeTupleStruct = SerializeList;
    type SerializeTupleVariant = SerializeList;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeMap;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Bool(v.into()))
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.try_into()?))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.try_into()?))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.into()))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        Ok(Value::String(v.into()))
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Ok(Value::List(
            v.iter().copied().map(|b| Value::Number(b.into())).collect(),
        ))
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
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
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
        let mut map = ValueMap::new();
        map.insert(variant.into(), value.serialize(self)?);
        Ok(Value::Map(map))
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SerializeList::new(len))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Ok(SerializeList::new(Some(len)))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Ok(SerializeList::new(Some(len)))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Ok(SerializeList::new_variant(variant, Some(len)))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(SerializeMap::new(len))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(SerializeMap::new(Some(len)))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Ok(SerializeMap::new_variant(variant, Some(len)))
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        Ok(Value::Number(v.into()))
    }

    fn collect_seq<I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        I: IntoIterator,
        <I as IntoIterator>::Item: Serialize,
    {
        Ok(Value::List(
            iter.into_iter()
                .map(|v| v.serialize(ValueSerializer))
                .collect::<Result<_, _>>()?,
        ))
    }

    fn collect_map<K, V, I>(self, iter: I) -> Result<Self::Ok, Self::Error>
    where
        K: Serialize,
        V: Serialize,
        I: IntoIterator<Item = (K, V)>,
    {
        Ok(Value::Map(
            iter.into_iter()
                .map(|(k, v)| -> Result<_, Self::Error> {
                    Ok((
                        k.serialize(ValueSerializer)?
                            .into_string()
                            .map_err(Error::NonStringKey)?,
                        v.serialize(ValueSerializer)?,
                    ))
                })
                .collect::<Result<_, _>>()?,
        ))
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Display,
    {
        Ok(Value::String(value.to_string().into()))
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

/// Serialize to a dices value
pub fn serialize_to_value<T: Serialize, II>(value: T) -> Result<Value<II>, Error> {
    value
        .serialize(ValueSerializer)
        .map(Value::with_arbitrary_injected_intrisics)
}

#[cfg(test)]
#[cfg(feature = "parse_value")]
mod tests {
    use serde::Serialize;
    use std::{fmt::Debug, str::FromStr};

    use crate::{intrisics::NoInjectedIntrisics, Value};

    use super::serialize_to_value;

    fn test_impl<T: Serialize + Debug>(value: T, expected: &str) {
        let serialized = serialize_to_value(&value).expect("Cannot serialize value");
        let expected = Value::<NoInjectedIntrisics>::from_str(expected.trim())
            .expect("Cannot parse the expected value");

        assert_eq!(
            serialized, expected,
            "The value {value:?} did not serialize as expected"
        )
    }

    macro_rules! tests {
        (
            $(
                $name:ident : $value:expr => $expected:literal ;
            )*
        ) => {
            $(
                #[test]
                fn $name() {
                    test_impl($value, $expected)
                }
            )*
        };
    }

    tests! {
        string: "Hello" => "\"Hello\"";
        slice: [1,2,3] => "[1,2,3]";
        struct_: {
            #[derive(Debug, serde::Serialize)]
            struct Hello {
                value: i64,
                what: String
            }
            Hello { value: 42, what: String::from("Answer") }
        } => "<| value: 42, what: \"Answer\" |>";
        struct_tuple: {
            #[derive(Debug, serde::Serialize)]
            struct Hello(i64, &'static str, bool);
            Hello (1,"Hello",true)
        } => "[1,\"Hello\",true]";
        u8_: 3u8 => "3"; i8_: -3i8 => "-3";
        u16_: 3u16 => "3"; i16_: -3i16 => "-3";
        u32_: 3u32 => "3"; i32_: -3i32 => "-3";
        u64_: 3u64 => "3"; i64_: -3i64 => "-3";
        u128_: 3u128 => "3"; i128_: -3i128 => "-3";
    }
}
