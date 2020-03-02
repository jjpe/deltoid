//! These integration testss exist because the Delta derive macro cannot be
//! used within the `struct-delta-trait` crate, where `RwLock` is defined.

#![allow(non_snake_case)]

use serde_json;
use struct_delta_derive;
use struct_delta_trait::*;

#[derive(Debug, Clone, PartialEq, struct_delta_derive::Delta)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
struct Foo {
    field0: String,
    field1: u8
}

#[test]
fn rwlock__serialize() {
    let value: RwLock<Foo> = RwLock::new(Foo {
        field0: "flapjacks are fun".to_string(),
        field1: 42,
    });
    println!("value: {:#?}", value);

    let serialized: String = serde_json::to_string(&value)
        .unwrap(/*TODO*/);
    println!("serialized:\n{}", serialized);

    let expected = r#"{"field0":"flapjacks are fun","field1":42}"#;
    println!("expected: {:#?}", expected);

    assert_eq!(serialized, expected);
}

#[test]
fn rwlock__deserialize() {
    let serialized = r#"{"field0":"flapjacks are fun","field1":42}"#;
    println!("serialized:\n{}", serialized);

    let deserialized: RwLock<Foo> = serde_json::from_str(&serialized)
        .unwrap(/*TODO*/);
    println!("deserialized: {:#?}", deserialized);

    let expected: RwLock<Foo> = RwLock::new(Foo {
        field0: "flapjacks are fun".to_string(),
        field1: 42,
    });
    println!("expected: {:#?}", expected);

    assert_eq!(deserialized, expected);
}

#[test]
fn rwlock__apply_delta() {
    let value0: RwLock<Foo> = RwLock::new(Foo {
        field0: "flapjacks are fun".to_string(),
        field1: 42,
    });
    println!("value0: {:#?}", value0);

    let delta: RwLockDelta<Foo> = RwLockDelta(Some(FooDelta {
        field0: Some(
            "flapjacks are fun?".to_string().into_delta().unwrap()
        ),
        field1: None,
    }));
    println!("delta: {:#?}", delta);

    let value1: RwLock<Foo> = value0.apply_delta(&delta).unwrap();
    println!("value1: {:#?}", value1);

    let expected: RwLock<Foo> = RwLock::new(Foo {
        field0: "flapjacks are fun?".to_string(),
        field1: 42,
    });
    println!("expected: {:#?}", expected);

    assert_eq!(value1, expected);
}

#[test]
fn rwlock__calculate_delta() {
    let value0: RwLock<Foo> = RwLock::new(Foo {
        field0: "flapjacks are fun".to_string(),
        field1: 42,
    });
    println!("value0: {:#?}", value0);

    let value1: RwLock<Foo> = RwLock::new(Foo {
        field0: "flapjacks are fun?".to_string(),
        field1: 42,
    });
    println!("value1: {:#?}", value1);

    let delta = value0.delta(&value1).unwrap();
    println!("delta: {:#?}", delta);

    let expected: RwLockDelta<Foo> = RwLockDelta(Some(FooDelta {
        field0: Some(
            "flapjacks are fun?".to_string().into_delta().unwrap()
        ),
        field1: None,
    }));
    println!("expected: {:#?}", expected);

    assert_eq!(delta, expected);
}
