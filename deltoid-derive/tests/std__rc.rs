//! Tests for the `std::rc` module
#![allow(non_snake_case)]

use std::rc::Rc;
use deltoid::{Deltoid, DeltaResult, RcDelta, StringDelta};
use deltoid_derive::Delta;
use serde_derive::{Deserialize, Serialize};


#[derive(Clone, Debug, PartialEq, Eq, Delta, Deserialize, Serialize)]
struct Foo {
    s: String,
    i: usize,
}

#[test]
fn Rc__calculate_delta() -> DeltaResult<()> {
    let v0 = Rc::new(Foo { s: "hello world".to_string(), i: 42 });
    let v1 = Rc::new(Foo { s: "hello world!!".to_string(), i: 42 });
    let delta0 = v0.delta(&v1)?;
    println!("delta0: {:#?}", delta0);
    let expected = RcDelta(Box::new(FooDelta {
        s: Some(StringDelta(Some("hello world!!".to_string()))),
        i: None,
    }));
    assert_eq!(delta0, expected, "{:#?}\n    !=\n{:#?}", delta0, expected);

    let v2 = v0.apply_delta(&delta0)?;
    println!("v2: {:#?}", v2);
    assert_eq!(v1, v2);

    let delta1 = v1.delta(&v0)?;
    println!("delta1: {:#?}", delta1);
    assert_eq!(delta1, RcDelta(Box::new(FooDelta {
        s: Some(StringDelta(Some("hello world".to_string()))),
        i: None,
    })));
    let v3 = v1.apply_delta(&delta1)?;
    println!("v3: {:#?}", v3);
    assert_eq!(v0, v3);

    Ok(())
}

#[test]
fn Rc__apply_delta() -> DeltaResult<()> {
    let v0 = Rc::new(Foo { s: "hello world".to_string(), i: 42 });
    let delta = RcDelta(Box::new(FooDelta {
        s: Some(StringDelta(Some("hello world!!".to_string()))),
        i: None,
    }));
    let v1 = v0.apply_delta(&delta)?;
    let expected = Rc::new(Foo { s: "hello world!!".to_string(), i: 42 });
    assert_eq!(expected, v1);

    Ok(())
}
