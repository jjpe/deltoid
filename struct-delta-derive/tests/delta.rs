//!
#![allow(non_snake_case)]

use struct_delta_trait::{DeltaOps, DeltaResult};
use struct_delta_derive::Delta;
use std::convert::{TryInto};
use std::borrow::{Cow};


// #[derive(Debug, PartialEq, Delta)]
// enum Qux {
//     Floof(u8, u64),
//     Blah { one: u8, two: () },
//     Flah { one: u16, two: String, three: u32 },
//     Gah, // TODO: unit structs
// }

#[derive(Debug, PartialEq)]
#[derive(Delta)]
enum Corge<Tx, U: Copy> {
    Quux,
    Grault(u8, Tx),
    #[allow(unused)] Floof { one: u8, two: Tx, three: U },
}


#[derive(Debug, PartialEq)]
#[derive(Delta)]
pub struct Foo<F: Copy> where F: Copy {
    f0: (),
    f1: F,
    f2: String,
}

#[derive(Debug, PartialEq)]
#[derive(Delta)]
pub struct Bar<S: Copy>(u8, S) where S: std::fmt::Debug;

#[derive(Debug, PartialEq)]
#[derive(Delta)]
pub struct Baz;

#[derive(Debug, PartialEq)]
#[derive(Delta)]
pub struct Plow(Cow<'static, String>);













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
        f1: Some(300.try_into()?),
        f2: Some("hello world!!!".try_into()?),
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
        f1: Some(300.try_into()?),
        f2: Some("hello world!!!".try_into()?),
    };
    let val1 = val0.apply_delta(&delta)?;
    let expected: Foo<u16> = Foo {
        f0: (),
        f1: 300,
        f2: String::from("hello world!!!")
    };
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}



#[test]
pub fn generic_tuple_struct__calculate_delta() -> DeltaResult<()> {
    let val0: Bar<u16> = Bar(42u8, 300u16);
    let val1: Bar<u16> = Bar(100u8, 300u16);
    let delta: BarDelta<u16> = val0.delta(&val1)?;
    let expected: BarDelta<u16> = BarDelta(Some(100.try_into()?), None);
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn generic_tuple_struct__apply_delta() -> DeltaResult<()>  {
    let val0: Bar<u16> = Bar(42u8, 300u16);
    let delta: BarDelta<u16> = BarDelta(Some(100.try_into()?), None);
    let val1: Bar<u16> = val0.apply_delta(&delta)?;
    let expected: Bar<u16> = Bar(100u8, 300u16);
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}

#[test]
pub fn generic_unit_struct__calculate_delta() -> DeltaResult<()> {
    let val0 = Baz;
    let val1 = Baz;
    let delta: BazDelta = val0.delta(&val1)?;
    let expected: BazDelta = BazDelta;
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn generic_unit_struct__apply_delta() -> DeltaResult<()>  {
    let val0 = Baz;
    let delta: BazDelta = BazDelta;
    let val1: Baz = val0.apply_delta(&delta)?;
    let expected = Baz;
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}
