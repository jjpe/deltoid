//! These integration tests exist because the Delta derive macro cannot be
//! used within the `struct-delta-trait` crate, where `RwLock` is defined.

#![allow(non_snake_case)]

#[allow(unused)] use deltoid::{
    Apply, Delta, DeltaResult, FromDelta, IntoDelta,
    ArcDelta, RwLock, RwLockDelta, StringDelta
};
use deltoid_derive::Delta;
use serde_json;
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Delta, Deserialize, Serialize)]
struct Foo {
    field0: String,
    field1: u8
}

#[test]
fn RwLock__serialize() {
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
fn RwLock__deserialize() {
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
fn RwLock__apply() {
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

    let value1: RwLock<Foo> = value0.apply(delta).unwrap();
    println!("value1: {:#?}", value1);

    let expected: RwLock<Foo> = RwLock::new(Foo {
        field0: "flapjacks are fun?".to_string(),
        field1: 42,
    });
    println!("expected: {:#?}", expected);

    assert_eq!(value1, expected);
}

#[test]
fn RwLock__calculate_delta() {
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





#[test]
fn Arc__calculate_delta() -> DeltaResult<()> {
    let v0 = Arc::new(Foo { field0: "hello world".to_string(),   field1: 42 });
    let v1 = Arc::new(Foo { field0: "hello world!!".to_string(), field1: 42 });
    let delta0 = v0.delta(&v1)?;
    println!("delta0: {:#?}", delta0);
    let expected = ArcDelta(Some(Box::new(FooDelta {
        field0: Some(StringDelta(Some("hello world!!".to_string()))),
        field1: None,
    })));
    assert_eq!(delta0, expected, "{:#?}\n    !=\n{:#?}", delta0, expected);

    let v2 = v0.apply(delta0)?;
    println!("v2: {:#?}", v2);
    assert_eq!(v1, v2);

    let delta1 = v1.delta(&v0)?;
    println!("delta1: {:#?}", delta1);
    assert_eq!(delta1, ArcDelta(Some(Box::new(FooDelta {
        field0: Some(StringDelta(Some("hello world".to_string()))),
        field1: None,
    }))));
    let v3 = v1.apply(delta1)?;
    println!("v3: {:#?}", v3);
    assert_eq!(v0, v3);

    Ok(())
}

#[test]
fn Arc__apply() -> DeltaResult<()> {
    let v0 = Arc::new(Foo { field0: "hello world".to_string(), field1: 42 });
    let delta = ArcDelta(Some(Box::new(FooDelta {
        field0: Some(StringDelta(Some("hello world!!".to_string()))),
        field1: None,
    })));
    let v1 = v0.apply(delta)?;
    let expected = Arc::new(Foo {
        field0: "hello world!!".to_string(),
        field1: 42
    });
    assert_eq!(expected, v1);

    Ok(())
}
