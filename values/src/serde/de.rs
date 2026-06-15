//! Deserialize Rust types out of a dices [`Value`].
//!
//! This is the inverse of [`crate::serde::ser`]: it accepts the same value
//! shapes that serializer produces, and is forgiving where it can be (a struct
//! may also be read from a list, integers are range-checked into the requested
//! width, and floats are read losslessly-where-possible from integers).
//!
//! Floating point numbers are read from a [`Value::Int`] (the value model has
//! no float kind), so a serialize/deserialize round trip through a float will
//! generally fail at serialization time, not here.

use num::{NumCast, ToPrimitive};
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, EnumAccess, IntoDeserializer, MapAccess, SeqAccess,
    VariantAccess, Visitor,
};
use snafu::OptionExt;

use super::error::{Error, IntegerOutOfRangeSnafu, Result};
use crate::Value;

/// Deserialize a value out of a [`Value`].
pub fn from_value<T: DeserializeOwned>(value: Value) -> Result<T> {
    T::deserialize(ValueDeserializer(value))
}

/// Deserialize a Rust type from a dices [`Value`].
pub struct ValueDeserializer(pub Value);

/// Implement an integer `deserialize_*` method via [`NumCast`].
macro_rules! deserialize_int {
    ($method:ident, $visit:ident, $ty:ty) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self.0 {
                Value::Int(int) => visitor.$visit(
                    <$ty as NumCast>::from(int.clone())
                        .context(IntegerOutOfRangeSnafu { found: int })?,
                ),
                other => Err(Error::unexpected("int", &other)),
            }
        }
    };
}

impl<'de> de::Deserializer<'de> for ValueDeserializer {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Null(_) => visitor.visit_unit(),
            Value::Bool(b) => visitor.visit_bool(b.get()),
            Value::Int(int) => {
                if let Some(v) = int.to_i64() {
                    visitor.visit_i64(v)
                } else if let Some(v) = int.to_u64() {
                    visitor.visit_u64(v)
                } else if let Some(v) = int.to_i128() {
                    visitor.visit_i128(v)
                } else if let Some(v) = int.to_u128() {
                    visitor.visit_u128(v)
                } else {
                    Err(Error::IntegerOutOfRange { found: int })
                }
            }
            Value::String(s) => visitor.visit_string(s.into()),
            Value::List(list) => visitor.visit_seq(SeqDeserializer {
                iter: list.into_iter(),
            }),
            Value::Map(map) => visitor.visit_map(MapDeserializer {
                iter: map.into_iter(),
                value: None,
            }),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Bool(b) => visitor.visit_bool(b.get()),
            other => Err(Error::unexpected("bool", &other)),
        }
    }

    deserialize_int!(deserialize_i8, visit_i8, i8);
    deserialize_int!(deserialize_i16, visit_i16, i16);
    deserialize_int!(deserialize_i32, visit_i32, i32);
    deserialize_int!(deserialize_i64, visit_i64, i64);
    deserialize_int!(deserialize_i128, visit_i128, i128);
    deserialize_int!(deserialize_u8, visit_u8, u8);
    deserialize_int!(deserialize_u16, visit_u16, u16);
    deserialize_int!(deserialize_u32, visit_u32, u32);
    deserialize_int!(deserialize_u64, visit_u64, u64);
    deserialize_int!(deserialize_u128, visit_u128, u128);

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Int(int) => visitor.visit_f32(
                int.to_f64()
                    .ok_or_else(|| Error::IntegerOutOfRange { found: int })? as f32,
            ),
            other => Err(Error::unexpected("int", &other)),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Int(int) => visitor.visit_f64(
                int.to_f64()
                    .ok_or_else(|| Error::IntegerOutOfRange { found: int })?,
            ),
            other => Err(Error::unexpected("int", &other)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => {
                let mut chars = s.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => visitor.visit_char(c),
                    _ => Err(Error::InvalidChar { found: s }),
                }
            }
            other => Err(Error::unexpected("string", &other)),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::String(s) => visitor.visit_string(s.into()),
            other => Err(Error::unexpected("string", &other)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            // A string deserializes to its raw bytes.
            Value::String(s) => visitor.visit_byte_buf(s.as_str().as_bytes().to_vec()),
            // A list of integers deserializes element-by-element into bytes.
            Value::List(list) => {
                let mut bytes = Vec::with_capacity(list.len());
                for value in list {
                    match value {
                        Value::Int(int) => bytes.push(
                            <u8 as NumCast>::from(int.clone())
                                .context(IntegerOutOfRangeSnafu { found: int })?,
                        ),
                        other => return Err(Error::unexpected("int", &other)),
                    }
                }
                visitor.visit_byte_buf(bytes)
            }
            other => Err(Error::unexpected("string or list", &other)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Null(_) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Null(_) => visitor.visit_unit(),
            other => Err(Error::unexpected("null", &other)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::List(list) => visitor.visit_seq(SeqDeserializer {
                iter: list.into_iter(),
            }),
            other => Err(Error::unexpected("list", &other)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            Value::Map(map) => visitor.visit_map(MapDeserializer {
                iter: map.into_iter(),
                value: None,
            }),
            other => Err(Error::unexpected("map", &other)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        // A struct can be read from a map (by field name) or a list (by
        // declaration order), mirroring what the serializer might have made.
        match self.0 {
            Value::Map(_) => self.deserialize_map(visitor),
            Value::List(_) => self.deserialize_seq(visitor),
            other => Err(Error::unexpected("map or list", &other)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.0 {
            // A bare string is a unit variant.
            Value::String(variant) => visitor.visit_enum(EnumDeserializer {
                variant,
                value: None,
            }),
            // A single-entry map is a variant carrying content.
            Value::Map(map) => {
                let mut iter = map.into_iter();
                let Some((variant, value)) = iter.next() else {
                    return Err(Error::InvalidEnum {
                        reason: "map must hold exactly one entry",
                    });
                };
                if iter.next().is_some() {
                    return Err(Error::InvalidEnum {
                        reason: "map must hold exactly one entry",
                    });
                }
                visitor.visit_enum(EnumDeserializer {
                    variant,
                    value: Some(value),
                })
            }
            other => Err(Error::unexpected("string or map", &other)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

/// [`SeqAccess`] over the elements of a [`Value::List`].
struct SeqDeserializer {
    iter: crate::list::IntoIter,
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(ValueDeserializer(value)).map(Some),
            None => Ok(None),
        }
    }
}

/// [`MapAccess`] over the entries of a [`Value::Map`].
struct MapDeserializer {
    iter: crate::map::IntoIter,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(MapKeyDeserializer(key)).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let value = self
            .value
            .take()
            .expect("next_value_seed called before next_key_seed");
        seed.deserialize(ValueDeserializer(value))
    }
}

/// Deserializer for a map key, which is always stored as a string.
///
/// Because the serializer coerces non-string keys (integers, booleans, ...) to
/// strings, this deserializer parses them back when an integer or boolean key
/// is requested, so maps with such keys round-trip seamlessly. Every other
/// request is forwarded to the underlying string [`ValueDeserializer`].
struct MapKeyDeserializer(crate::string::ValueString);

impl MapKeyDeserializer {
    /// The key seen as a plain string value deserializer.
    fn into_string_de(self) -> ValueDeserializer {
        ValueDeserializer(Value::String(self.0))
    }
}

/// Parse a map key string into a scalar, erroring with a clear message.
macro_rules! deserialize_key_parsed {
    ($method:ident, $visit:ident, $ty:ty) => {
        fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
        where
            V: Visitor<'de>,
        {
            match self.0.as_str().parse::<$ty>() {
                Ok(v) => visitor.$visit(v),
                Err(_) => Err(Error::Custom {
                    message: format!(
                        "map key {:?} is not a valid {}",
                        self.0.as_str(),
                        stringify!($ty)
                    ),
                }),
            }
        }
    };
}

/// Forward a uniform `(self, visitor)` method to the string deserializer.
macro_rules! delegate_key {
    ($($method:ident),* $(,)?) => {
        $(
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                de::Deserializer::$method(self.into_string_de(), visitor)
            }
        )*
    };
}

impl<'de> de::Deserializer<'de> for MapKeyDeserializer {
    type Error = Error;

    deserialize_key_parsed!(deserialize_bool, visit_bool, bool);
    deserialize_key_parsed!(deserialize_i8, visit_i8, i8);
    deserialize_key_parsed!(deserialize_i16, visit_i16, i16);
    deserialize_key_parsed!(deserialize_i32, visit_i32, i32);
    deserialize_key_parsed!(deserialize_i64, visit_i64, i64);
    deserialize_key_parsed!(deserialize_i128, visit_i128, i128);
    deserialize_key_parsed!(deserialize_u8, visit_u8, u8);
    deserialize_key_parsed!(deserialize_u16, visit_u16, u16);
    deserialize_key_parsed!(deserialize_u32, visit_u32, u32);
    deserialize_key_parsed!(deserialize_u64, visit_u64, u64);
    deserialize_key_parsed!(deserialize_u128, visit_u128, u128);

    delegate_key!(
        deserialize_any,
        deserialize_f32,
        deserialize_f64,
        deserialize_char,
        deserialize_str,
        deserialize_string,
        deserialize_bytes,
        deserialize_byte_buf,
        deserialize_option,
        deserialize_unit,
        deserialize_seq,
        deserialize_map,
        deserialize_identifier,
        deserialize_ignored_any,
    );

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_string_de().deserialize_unit_struct(name, visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_string_de()
            .deserialize_newtype_struct(name, visitor)
    }

    fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_string_de().deserialize_tuple(len, visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_string_de()
            .deserialize_tuple_struct(name, len, visitor)
    }

    fn deserialize_struct<V>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_string_de()
            .deserialize_struct(name, fields, visitor)
    }

    fn deserialize_enum<V>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.into_string_de()
            .deserialize_enum(name, variants, visitor)
    }
}

/// [`EnumAccess`] for an externally-tagged enum value.
struct EnumDeserializer {
    variant: crate::string::ValueString,
    value: Option<Value>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = seed.deserialize(self.variant.as_str().into_deserializer())?;
        Ok((variant, VariantDeserializer { value: self.value }))
    }
}

/// [`VariantAccess`] for the content of an enum variant.
struct VariantDeserializer {
    value: Option<Value>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        match self.value {
            None => Ok(()),
            Some(_) => Err(Error::InvalidEnum {
                reason: "unit variant must not carry content",
            }),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let value = self.value.ok_or(Error::InvalidEnum {
            reason: "newtype variant is missing its content",
        })?;
        seed.deserialize(ValueDeserializer(value))
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.value.ok_or(Error::InvalidEnum {
            reason: "tuple variant is missing its content",
        })?;
        de::Deserializer::deserialize_seq(ValueDeserializer(value), visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        let value = self.value.ok_or(Error::InvalidEnum {
            reason: "struct variant is missing its content",
        })?;
        de::Deserializer::deserialize_map(ValueDeserializer(value), visitor)
    }
}
