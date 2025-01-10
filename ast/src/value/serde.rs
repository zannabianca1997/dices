use std::collections::BTreeMap;

use num_bigint::{BigInt, Sign};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::{
    ident::IdentStr,
    intrisics::{InjectedIntr, Intrisic},
};

use super::{
    Value, ValueBool, ValueClosure, ValueIntrisic, ValueList, ValueMap, ValueNull, ValueNumber,
    ValueString,
};

#[derive(Deserialize)]
#[serde(bound = "InjectedIntrisic: InjectedIntr", tag = "$type")]
/// Serialized for of a [`Value`]
enum Serialized<InjectedIntrisic> {
    #[serde(rename = "map")]
    NestedMap {
        #[serde(rename = "$content")]
        content: BTreeMap<ValueString, Value<InjectedIntrisic>>,
    },
    #[serde(rename = "intrisic")]
    NestedIntrisic {
        #[serde(rename = "$intrisic")]
        intrisic: Intrisic<InjectedIntrisic>,
    },
    #[serde(rename = "closure")]
    NestedClosure {
        #[serde(rename = "$params")]
        params: Box<[Box<IdentStr>]>,
        #[serde(rename = "$captures", default)]
        captures: BTreeMap<Box<IdentStr>, Value<InjectedIntrisic>>,
        #[serde(rename = "$body")]
        body: ByteBuf,
    },
    #[serde(rename = "number")]
    NestedNumber {
        #[serde(rename = "$sign")]
        sign: Sign,
        #[serde(rename = "$bytes")]
        bytes: ByteBuf,
    },

    #[serde(untagged)]
    Number(i64),
    #[serde(untagged)]
    Null(ValueNull),
    #[serde(untagged)]
    Bool(ValueBool),
    #[serde(untagged)]
    String(ValueString),
    #[serde(untagged)]
    List(ValueList<InjectedIntrisic>),
    #[serde(untagged)]
    Map(BTreeMap<ValueString, Value<InjectedIntrisic>>),
}

#[derive(Serialize)]
#[serde(bound = "InjectedIntrisic: InjectedIntr", tag = "$type")]
enum BorrowedSerialized<'m, InjectedIntrisic> {
    #[serde(rename = "map")]
    NestedMap {
        #[serde(rename = "$content")]
        content: &'m BTreeMap<ValueString, Value<InjectedIntrisic>>,
    },
    #[serde(rename = "intrisic")]
    NestedIntrisic {
        #[serde(rename = "$intrisic")]
        intrinsic: &'m Intrisic<InjectedIntrisic>,
    },
    #[serde(rename = "closure")]
    NestedClosure {
        #[serde(rename = "$params")]
        params: &'m [Box<IdentStr>],
        #[serde(rename = "$captures", skip_serializing_if = "BTreeMap::is_empty")]
        captures: &'m BTreeMap<Box<IdentStr>, Value<InjectedIntrisic>>,
        #[serde(rename = "$body")]
        body: ByteBuf,
    },
    #[serde(rename = "number")]
    NestedNumber {
        #[serde(rename = "$sign")]
        sign: Sign,
        #[serde(rename = "$bytes")]
        bytes: ByteBuf,
    },

    #[serde(untagged)]
    Number(i64),
    #[serde(untagged)]
    Null(&'m ValueNull),
    #[serde(untagged)]
    Bool(&'m ValueBool),
    #[serde(untagged)]
    String(&'m ValueString),
    #[serde(untagged)]
    List(&'m ValueList<InjectedIntrisic>),
    #[serde(untagged)]
    Map(&'m BTreeMap<ValueString, Value<InjectedIntrisic>>),
}

impl<II> Serialize for Value<II>
where
    II: InjectedIntr,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Null(value_null) => BorrowedSerialized::Null(value_null),
            Value::Bool(value_bool) => BorrowedSerialized::Bool(value_bool),
            Value::Number(value_number) => match i64::try_from(&value_number.0) {
                Ok(small) => BorrowedSerialized::Number(small),
                Err(_) => {
                    let (sign, bytes) = value_number.0.to_bytes_le();
                    BorrowedSerialized::NestedNumber {
                        sign,
                        bytes: ByteBuf::from(bytes),
                    }
                }
            },
            Value::String(value_string) => BorrowedSerialized::String(value_string),
            Value::List(value_list) => BorrowedSerialized::List(value_list),
            Value::Map(value_map) => {
                if value_map.contains("$type") {
                    BorrowedSerialized::NestedMap {
                        content: &value_map.0,
                    }
                } else {
                    BorrowedSerialized::Map(&value_map.0)
                }
            }
            Value::Intrisic(ValueIntrisic(intrinsic)) => {
                BorrowedSerialized::NestedIntrisic { intrinsic }
            }
            Value::Closure(box ValueClosure {
                params,
                captures,
                body,
            }) => BorrowedSerialized::NestedClosure {
                params,
                captures,
                body: ByteBuf::from(
                    bincode::encode_to_vec(body, bincode::config::standard())
                        .map_err(<S::Error as serde::ser::Error>::custom)?,
                ),
            },
        }
        .serialize(serializer)
    }
}
impl<'de, II> Deserialize<'de> for Value<II>
where
    II: InjectedIntr,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(match Serialized::deserialize(deserializer)? {
            Serialized::NestedMap { content } | Serialized::Map(content) => {
                Value::Map(ValueMap(content))
            }
            Serialized::NestedIntrisic { intrisic } => Value::Intrisic(ValueIntrisic(intrisic)),
            Serialized::NestedClosure {
                params,
                captures,
                body,
            } => Value::Closure(Box::new(ValueClosure {
                params,
                captures,
                body: bincode::decode_from_slice(&body, bincode::config::standard())
                    .map_err(<D::Error as serde::de::Error>::custom)?
                    .0,
            })),
            Serialized::Null(value_null) => Value::Null(value_null),
            Serialized::Bool(value_bool) => Value::Bool(value_bool),
            Serialized::Number(value_number) => Value::Number(value_number.into()),
            Serialized::String(value_string) => Value::String(value_string),
            Serialized::List(value_list) => Value::List(value_list),
            Serialized::NestedNumber { sign, bytes } => {
                Value::Number(ValueNumber(BigInt::from_bytes_le(sign, &bytes)))
            }
        })
    }
}

mod serialize_to_value;
pub use serialize_to_value::{serialize_to_value, Error as SerializeToValueError};

mod deserialize_from_value;
pub use deserialize_from_value::{deserialize_from_value, Error as DeserializeFromValueError};
