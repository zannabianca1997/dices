//! Round-trip tests for the dices value (de)serializers.
//!
//! Each test maps a Rust value into a [`Value`] and back, asserting the value
//! survives the round trip and, where the shape matters, that the intermediate
//! [`Value`] is what we expect.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::{de::ValueDeserializer, error::Error, ser::ValueSerializer};
use crate::{Value, string::ValueString};

fn from_value<T: DeserializeOwned>(value: Value) -> T {
    super::from_value(value).expect("Failed deserialization")
}
fn to_value<T: Serialize>(value: &T) -> Value {
    super::to_value(value).expect("Failed serialization")
}

/// Assert a value survives a serialize/deserialize round trip unchanged.
fn assert_roundtrip<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let serialized = to_value(&value);
    let back: T = from_value(serialized);
    assert_eq!(value, back);
}

#[test]
fn primitives() {
    assert_roundtrip(true);
    assert_roundtrip(false);
    assert_roundtrip(0_i8);
    assert_roundtrip(-128_i8);
    assert_roundtrip(i64::MIN);
    assert_roundtrip(u64::MAX);
    assert_roundtrip(i128::MIN);
    assert_roundtrip(u128::MAX);
    assert_roundtrip('q');
    assert_roundtrip("a string".to_owned());
    assert_roundtrip(());
}

#[test]
fn scalar_value_shapes() {
    assert_eq!(to_value(&true), Value::Bool(true.into()));
    assert_eq!(
        to_value(&"hi"),
        Value::String(ValueString::new_static("hi"))
    );
    assert!(matches!(to_value(&7_u32), Value::Int(_)));
    assert_eq!(to_value(&()), Value::Null(crate::null::ValueNull));
}

#[test]
fn options() {
    assert_roundtrip(Some(42_i32));
    assert_roundtrip(Option::<i32>::None);
    assert_roundtrip(Some("hello".to_owned()));
    // None maps to null.
    assert_eq!(
        to_value(&Option::<i32>::None),
        Value::Null(crate::null::ValueNull)
    );
}

#[test]
fn sequences() {
    assert_roundtrip(vec![1_i32, 2, 3]);
    assert_roundtrip(Vec::<String>::new());
    assert_roundtrip((1_i32, "two".to_owned(), false));
    assert_roundtrip(vec![vec![1_u8, 2], vec![3]]);

    // A sequence is a list.
    assert!(matches!(to_value(&vec![1, 2, 3]), Value::List(_)));
}

#[test]
fn maps() {
    let mut map = BTreeMap::new();
    map.insert("alpha".to_owned(), 1_i32);
    map.insert("beta".to_owned(), 2);
    assert_roundtrip(map);

    // Integer keys are coerced to strings, which still round-trips.
    let mut int_keyed = BTreeMap::new();
    int_keyed.insert(1_u32, "one".to_owned());
    int_keyed.insert(2, "two".to_owned());
    assert_roundtrip(int_keyed);
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Point {
    x: i32,
    y: i32,
    label: String,
}

#[test]
fn structs() {
    let point = Point {
        x: 3,
        y: -4,
        label: "origin-ish".to_owned(),
    };
    assert_roundtrip(point);

    // A struct serializes to a map keyed by field name.
    let value = to_value(&Point {
        x: 1,
        y: 2,
        label: "p".to_owned(),
    });
    let Value::Map(map) = value else {
        panic!("expected a map, got {value:?}");
    };
    assert_eq!(map.get("x"), Some(&to_value(&1_i32)));
    assert!(map.contains_key("y"));
    assert!(map.contains_key("label"));
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Wrapper(i32);

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Unit;

#[test]
fn newtype_and_unit_structs() {
    // Newtype structs are transparent.
    assert_roundtrip(Wrapper(99));
    assert_eq!(to_value(&Wrapper(99)), to_value(&99_i32));

    // Unit structs map to null.
    assert_roundtrip(Unit);
    assert_eq!(to_value(&Unit), Value::Null(crate::null::ValueNull));
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
enum Shape {
    Empty,
    Radius(i32),
    Pair(i32, i32),
    Named { width: i32, height: i32 },
}

#[test]
fn enums() {
    // Unit variant -> bare string.
    assert_roundtrip(Shape::Empty);
    assert_eq!(
        to_value(&Shape::Empty),
        Value::String(ValueString::new_static("Empty"))
    );

    // Newtype variant -> { variant: value }.
    assert_roundtrip(Shape::Radius(5));

    // Tuple variant -> { variant: [..] }.
    assert_roundtrip(Shape::Pair(1, 2));

    // Struct variant -> { variant: { .. } }.
    assert_roundtrip(Shape::Named {
        width: 10,
        height: 20,
    });
}

#[test]
fn deserialize_any_via_untyped() {
    // Untyped deserialization should work through `deserialize_any`.
    #[derive(Debug, PartialEq, Deserialize)]
    #[serde(untagged)]
    enum Any {
        Bool(bool),
        Int(i64),
        Text(String),
        List(Vec<Any>),
    }

    // A list of mixed scalars, built by serializing a heterogeneous tuple.
    let value = to_value(&(true, 7_i64, "x".to_owned()));
    let any: Any = from_value(value);
    assert_eq!(
        any,
        Any::List(vec![
            Any::Bool(true),
            Any::Int(7),
            Any::Text("x".to_owned())
        ])
    );
}

#[test]
fn float_serialization_is_rejected() {
    let err = 1.5_f64.serialize(ValueSerializer).unwrap_err();
    assert_eq!(err, Error::FloatUnsupported);
}

#[test]
fn wrong_type_is_an_error() {
    // Asking for a bool from a string should fail with a typed error.
    let err = bool::deserialize(ValueDeserializer(Value::String(ValueString::new_static(
        "no",
    ))))
    .unwrap_err();
    assert!(matches!(err, Error::UnexpectedType { .. }));
}

#[test]
fn integer_out_of_range_is_an_error() {
    // 300 does not fit in a u8.
    let value = to_value(&300_i32);
    let err = u8::deserialize(ValueDeserializer(value)).unwrap_err();
    assert!(matches!(err, Error::IntegerOutOfRange { .. }));
}
