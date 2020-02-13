//!
#![allow(non_snake_case)]

use struct_delta_trait::{Delta, DeltaError, DeltaResult};
use serde_json::{json, Value as JsonValue};


#[derive(Debug, PartialEq, struct_delta_derive::Delta)]
pub struct Foo<T: Copy>
where T: Copy {
    f0: (),
    f1: T,
    f2: String,
}

#[derive(Debug, PartialEq, struct_delta_derive::Delta)]
pub struct Bar<T: Copy>(u8, T) where T: std::fmt::Debug;

#[derive(Debug, PartialEq, struct_delta_derive::Delta)]
pub struct Baz;


#[test]
pub fn generic_struct__calculate_delta() -> DeltaResult<()> {
    let val0: Foo<u16> = Foo {
        f0: (),
        f1: 42 as u16,
        f2: "hello world".into()
    };
    let val1: Foo<u16> = Foo {
        f0: (),
        f1: 300,
        f2: "hello world!!!".into()
    };
    let delta: FooDelta<u16> = val0.delta(&val1)?;
    let expected: FooDelta<u16> = FooDelta {
        f0: None,
        f1: Some(300),
        f2: Some("hello world!!!".into()),
    };
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn generic_struct__apply_delta() -> DeltaResult<()>  {
    let val0: Foo<u16> = Foo {
        f0: (),
        f1: 42 as u16,
        f2: "hello world".into()
    };
    let delta: FooDelta<u16> = FooDelta {
        f0: None,
        f1: Some(300),
        f2: Some("hello world!!!".into()),
    };
    let val1 = val0.apply_delta(&delta)?;
    let expected: Foo<u16> = Foo {
        f0: (),
        f1: 300,
        f2: "hello world!!!".into()
    };
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}



#[test]
pub fn generic_tuple_struct__calculate_delta() -> DeltaResult<()> {
    let val0: Bar<u16> = Bar(42u8, 300u16);
    let val1: Bar<u16> = Bar(100u8, 300u16);
    let delta: BarDelta<u16> = val0.delta(&val1)?;
    let expected: BarDelta<u16> = BarDelta(Some(100), None);
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn generic_tuple_struct__apply_delta() -> DeltaResult<()>  {
    let val0: Bar<u16> = Bar(42u8, 300u16);
    let delta: BarDelta<u16> = BarDelta(Some(100), None);
    let val1: Bar<u16> = val0.apply_delta(&delta)?;
    let expected: Bar<u16> = Bar(100u8, 300u16);
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}
