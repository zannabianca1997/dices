//! End-to-end tests for the `Injectable` derive and `#[injectable]` attribute.
#![cfg(feature = "macros")]

use std::any::Any;

use dices_values::{
    Injectable, Value,
    identifier::Identifier,
    injectable,
    injected::{
        ValueInjected,
        call::{Callable, InjectedContext},
    },
    int::ValueInt,
    string::ValueString,
};
use serde::{Deserialize, Serialize};

/// A minimal context that does nothing useful — enough to call functions that
/// ignore their context.
struct DummyCx;

impl InjectedContext for DummyCx {
    fn dice(&mut self, faces: ValueInt) -> ValueInt {
        faces
    }
    fn enter_scope(&mut self) -> Box<dyn Any> {
        Box::new(())
    }
    fn exit_scope(&mut self, _data: Box<dyn Any>) {}
    fn enter_jail(&mut self) -> Box<dyn Any> {
        Box::new(())
    }
    fn exit_jail(&mut self, _data: Box<dyn Any>) {}
    fn let_var(&mut self, _name: Identifier, _value: Value) {}
    fn var(&self, _name: &Identifier) -> Option<&Value> {
        None
    }
    fn var_mut(&mut self, _name: &Identifier) -> Option<&mut Value> {
        None
    }
}

fn int(n: i64) -> Value {
    dices_values::serde::to_value(&n).unwrap()
}

// --- Attribute macro: TryFrom args + TryInto return --------------------------

/// Adds two integers.
#[injectable]
fn Add(#[cx] _cx: &mut dyn InjectedContext, a: ValueInt, b: ValueInt) -> ValueInt {
    a + b
}

#[test]
fn callable_with_tryfrom() {
    let mut cx = DummyCx;
    let result = Add.call(&mut cx, &[int(3), int(4)]).unwrap();
    assert_eq!(result, int(7));
}

#[test]
fn callable_wrong_arg_count() {
    let mut cx = DummyCx;
    let err = Add.call(&mut cx, &[int(3)]).unwrap_err();
    assert!(err.to_string().contains("expected 2"), "{err}");
}

// --- Attribute macro: serde-fallback args and return -------------------------

#[derive(Debug, Serialize, Deserialize)]
struct Point {
    x: i64,
    y: i64,
}

/// Sums the coordinates of a point.
#[injectable]
fn ManhattanNorm(p: Point) -> i64 {
    p.x + p.y
}

#[test]
fn callable_with_serde_fallback() {
    let point = dices_values::serde::to_value(&Point { x: 2, y: 5 }).unwrap();
    let mut cx = DummyCx;
    let result = ManhattanNorm.call(&mut cx, &[point]).unwrap();
    // `i64` return converts via the serde fallback (no `TryInto<Value>`).
    assert_eq!(result, int(7));
}

// --- Derive macro ------------------------------------------------------------

/// A bundle of two callables.
#[derive(Injectable, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Std {
    add: Add,
    norm: ManhattanNorm,
}

#[test]
fn derive_reads_as_map() {
    let value = ValueInjected::new(Std {
        add: Add,
        norm: ManhattanNorm,
    })
    .read()
    .unwrap();

    let Value::Map(map) = value else {
        panic!("expected a map, got {value:?}");
    };

    assert_eq!(map.len(), 2);
    let add = map.get(&ValueString::new_static("add")).unwrap();
    let norm = map.get(&ValueString::new_static("norm")).unwrap();
    assert!(matches!(add, Value::Injected(_)));
    assert!(matches!(norm, Value::Injected(_)));
}

#[test]
fn derive_description_from_doc() {
    let injected = ValueInjected::new(Std {
        add: Add,
        norm: ManhattanNorm,
    });
    assert_eq!(
        injected.description().to_string(),
        "A bundle of two callables."
    );
}
