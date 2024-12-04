/**
 * Deserializing from a `dices` value
 *
 * A lot of this code is heavily inspired (and in some part straight up copied) from
 * the eccellent work of [dtolnay](https://github.com/dtolnay) in his library `serde_json`
 */
use derive_more::derive::{Display, Error, From};
use num_bigint::BigInt;
use serde::{
    de::{
        DeserializeSeed, EnumAccess, Expected, IntoDeserializer, MapAccess, SeqAccess, Unexpected,
        VariantAccess, Visitor,
    },
    Deserialize, Deserializer,
};
use std::fmt::Display;

use crate::{
    intrisics::{InjectedIntr, NoInjectedIntrisics},
    value::{ValueList, ValueMap, ValueNumber, ValueString},
    Value,
};

use super::serialize_to_value;

#[derive(Debug, Display, Error, From, Clone)]
pub enum Error {
    #[display("{_0}")]
    Custom(#[error(not(source))] String),
    #[display("Number {_0} is too big for serde data model")]
    NumberTooBig(#[error(not(source))] BigInt),
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

pub fn deserialize_from_value<'de, T: Deserialize<'de>, II: InjectedIntr>(
    value: Value<II>,
) -> Result<T, Error> {
    T::deserialize(value)
}

macro_rules! deserialize_numbers {
    (
        $method:ident , $typ:ty , $visit:ident , $fallback:ident ; $($rest:tt)*
    ) => {
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self {
                    Value::Number(v) => match <$typ>::try_from(v) {
                        Ok(v) => visitor.$visit(v),
                        Err(n) => Value::<NoInjectedIntrisics>::Number(n.into_original().into())
                            .$fallback(visitor),
                    },
                    _ => Err(self.invalid_type(&visitor)),
                }
            }

            deserialize_numbers!{
                $($rest)*
            }
    };
    (
        $method:ident , $typ:ty , $visit:ident ; $($rest:tt)*
    ) => {
            fn $method<V>(self, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: serde::de::Visitor<'de>,
            {
                match self {
                    Value::Number(v) => match <$typ>::try_from(v) {
                        Ok(v) => visitor.$visit(v),
                        Err(v) => Err(Value::<NoInjectedIntrisics>::Number(v.into_original().into()).invalid_type(&visitor)),
                    },
                    _ => Err(self.invalid_type(&visitor)),
                }
            }

            deserialize_numbers!{
                $($rest)*
            }
    };
    () => {};
}

impl<'de, II: InjectedIntr> Deserializer<'de> for Value<II> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Null(_) => visitor.visit_unit(),
            Value::Bool(v) => visitor.visit_bool(*v),
            Value::Number(n) => visit_number(n, visitor),
            Value::String(v) => visitor.visit_string(v.into()),
            Value::List(v) => visit_list(v, visitor),
            Value::Map(v) => visit_map(v, visitor),

            Value::Intrisic(_) | Value::Closure(_) => {
                // for strange values, we serialize them as plain ones, then try to deserialize from that
                let plain: Value<NoInjectedIntrisics> = serialize_to_value(&self)
                    .expect("Values should always be serializable to plain values");
                plain.deserialize_any(visitor)
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Bool(v) => visitor.visit_bool(*v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    deserialize_numbers! {
        deserialize_u8, u8, visit_u8, deserialize_i8;
        deserialize_i8, i8, visit_i8, deserialize_u16;
        deserialize_u16, u16, visit_u16, deserialize_i16;
        deserialize_i16, i16, visit_i16, deserialize_u32;
        deserialize_u32, u32, visit_u32, deserialize_i32;
        deserialize_i32, i32, visit_i32, deserialize_u64;
        deserialize_u64, u64, visit_u64, deserialize_i64;
        deserialize_i64, i64, visit_i64, deserialize_u128;
        deserialize_u128, u128, visit_u128, deserialize_i128;
        deserialize_i128, i128, visit_i128;
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::String(value_string) if value_string.chars().count() == 1 => {
                let ch = value_string.chars().next().unwrap();
                visitor.visit_char(ch)
            }
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::String(value_string) => visitor.visit_str(&value_string),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::String(value_string) => visitor.visit_string(value_string.into()),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::String(value_string) => visitor.visit_string(value_string.into()),
            Value::List(value_list) => visit_list(value_list, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::String(value_string) => visitor.visit_string(value_string.into()),
            Value::List(value_list) => visit_list(value_list, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Null(_) => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Null(_) => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::List(v) => visit_list(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
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
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::Map(v) => visit_map(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        match self {
            Value::List(v) => visit_list(v, visitor),
            Value::Map(v) => visit_map(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        let (variant, value) = match self {
            Value::Map(value) => {
                let mut iter = value.into_iter();
                let (variant, value) = match iter.next() {
                    Some(v) => v,
                    None => {
                        return Err(serde::de::Error::invalid_value(
                            Unexpected::Map,
                            &"map with a single key",
                        ));
                    }
                };
                // enums are encoded in json as maps with a single key:value pair
                if iter.next().is_some() {
                    return Err(serde::de::Error::invalid_value(
                        Unexpected::Map,
                        &"map with a single key",
                    ));
                }
                (variant, Some(value))
            }
            Value::String(variant) => (variant, None),
            other => {
                return Err(serde::de::Error::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: serde::de::Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

struct MapDeserializer<II> {
    iter: <ValueMap<II> as IntoIterator>::IntoIter,
    value: Option<Value<II>>,
}

impl<II> MapDeserializer<II> {
    fn new(map: ValueMap<II>) -> Self {
        MapDeserializer {
            iter: map.into_iter(),
            value: None,
        }
    }
}

impl<'de, II: InjectedIntr> MapAccess<'de> for MapDeserializer<II> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                self.value = Some(value);
                seed.deserialize(Value::<NoInjectedIntrisics>::String(key))
                    .map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value.take() {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::custom("value is missing")),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

fn visit_map<'de, II: InjectedIntr, V>(v: ValueMap<II>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = v.len();
    let mut deserializer = MapDeserializer::new(v);
    let map = visitor.visit_map(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(map)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in map",
        ))
    }
}

struct SeqDeserializer<II> {
    iter: <ValueList<II> as IntoIterator>::IntoIter,
}

impl<II> SeqDeserializer<II> {
    fn new(list: ValueList<II>) -> Self {
        SeqDeserializer {
            iter: list.into_iter(),
        }
    }
}

impl<'de, II: InjectedIntr> SeqAccess<'de> for SeqDeserializer<II> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

fn visit_list<'de, II: InjectedIntr, V>(v: ValueList<II>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = v.len();
    let mut deserializer = SeqDeserializer::new(v);
    let seq: V::Value = visitor.visit_seq(&mut deserializer)?;
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

fn visit_number<'de, V: Visitor<'de>>(n: ValueNumber, visitor: V) -> Result<V::Value, Error> {
    let n = match u64::try_from(n) {
        Ok(n) => return visitor.visit_u64(n),
        Err(n) => n.into_original(),
    };
    let n = match i64::try_from(n) {
        Ok(n) => return visitor.visit_i64(n),
        Err(n) => n.into_original(),
    };
    let n = match u128::try_from(n) {
        Ok(n) => return visitor.visit_u128(n),
        Err(n) => n.into_original(),
    };
    let n = match i128::try_from(n) {
        Ok(n) => return visitor.visit_i128(n),
        Err(n) => n.into_original(),
    };
    // No integer fit
    Err(Error::NumberTooBig(n))
}

impl<II> Value<II> {
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected {
        match self {
            Value::Null(_) => Unexpected::Unit,
            Value::Bool(b) => Unexpected::Bool(**b),
            Value::Number(n) => {
                let n = match u64::try_from(n.clone()) {
                    Ok(n) => return Unexpected::Unsigned(n),
                    Err(n) => n.into_original(),
                };
                let n = match i64::try_from(n.clone()) {
                    Ok(n) => return Unexpected::Signed(n),
                    Err(n) => n.into_original(),
                };
                if n < BigInt::ZERO {
                    Unexpected::Other("big negative number")
                } else {
                    Unexpected::Other("big positive number")
                }
            }
            Value::String(s) => Unexpected::Str(s),
            Value::List(_) => Unexpected::Seq,
            Value::Map(_) => Unexpected::Map,
            Value::Intrisic(_) => Unexpected::Other("intrisic"),
            Value::Closure(_) => Unexpected::Other("closure"),
        }
    }
}

struct EnumDeserializer<II> {
    variant: ValueString,
    value: Option<Value<II>>,
}

impl<'de, II: InjectedIntr> EnumAccess<'de> for EnumDeserializer<II> {
    type Error = Error;
    type Variant = VariantDeserializer<II>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer<II>), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantDeserializer<II> {
    value: Option<Value<II>>,
}

impl<'de, II: InjectedIntr> VariantAccess<'de> for VariantDeserializer<II> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::List(v)) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    visit_list(v, visitor)
                }
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(Value::Map(v)) => visit_map(v, visitor),
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

impl IntoDeserializer<'_, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}
