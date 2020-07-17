//!
#![allow(non_snake_case)]

#[allow(unused)] use deltoid::{
    Core, Apply, Delta, DeltaResult, FromDelta, IntoDelta,
    BoolDelta, StringDelta, U8Delta, UnitDelta,
};
use deltoid_derive::Delta;
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;


#[derive(Clone, Debug, PartialEq, Delta, Deserialize, Serialize)]
pub enum Qux1<T: Default> {
    Blah { #[delta(ignore_field)] one: u8, two: () },
    Floof(u8, T),
    Flah { one: Box<Qux2<(), ()>> },
    Gah,
}

#[derive(Clone, Debug, PartialEq, Delta, Deserialize, Serialize)]
pub enum Qux2<T, U: Default> {
    Floof(#[delta(ignore_field)] u8, T),
    Blah { #[delta(ignore_field)] one: u8, two: U },
    Flah { one: Box<Qux1<()>> },
    Gah,
}

#[derive(Clone, Debug, PartialEq, Delta, Deserialize, Serialize)]
enum Corge<T, U: Debug> {
    Quux,
    Grault(u8, T),
    Floof {
        one: u8,
        two: T,
        three: U
    },
}

#[derive(Clone, Debug, PartialEq, Default, Delta, Deserialize, Serialize)]
pub struct Foo0<F: Copy> where F: Copy {
    #[delta(ignore_field)]
    f0: (),
    f1: F,
    f2: String,
}

#[derive(Clone, Debug, PartialEq, Default, Delta, Deserialize, Serialize)]
pub struct Bar<S: Copy>(u8, S)
where S: std::fmt::Debug + Default;

#[derive(Clone, Debug, PartialEq, Default, Delta, Deserialize, Serialize)]
pub struct Baz;

#[derive(Clone, Debug, PartialEq, Default, Delta, Deserialize, Serialize)]
pub struct Plow(std::borrow::Cow<'static, String>);






#[test]
pub fn enum_struct_variant__delta__same_values() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Blah { one: 42u8, two: () };
    let val1: Qux2<String, ()> = Qux2::Blah { one: 42u8, two: () };
    let delta = val0.delta(&val1)?;
    let expected = Qux2Delta::Blah { one: std::marker::PhantomData, two: None };
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn enum_struct_variant__apply__same_values() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Blah { one: 42u8, two: () };
    let delta = Qux2Delta::Blah { one: std::marker::PhantomData, two: None };
    let val1 = val0.apply(delta)?;
    let expected: Qux2<String, ()> = Qux2::Blah { one: 42u8, two: () };
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}

#[test]
pub fn enum_struct_variant__delta__different_values() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Blah { one: 42u8, two: () };
    let val1: Qux2<String, ()> = Qux2::Blah { one: 100u8, two: () };
    let delta = val0.delta(&val1)?;
    let expected = Qux2Delta::Blah {
        one: std::marker::PhantomData,
        two: None
    };
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn enum_struct_variant__apply__different_values() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Blah { one: 42u8, two: () };
    let delta = Qux2Delta::Blah {
        one: std::marker::PhantomData,
        two: None
    };
    let val1 = val0.apply(delta)?;
    let expected: Qux2<String, ()> = Qux2::Blah { one: 42u8, two: () };
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}




#[test]
pub fn enum_tuple_struct_variant__delta__same_values() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Floof(42, String::from("foo"));
    let val1: Qux2<String, ()> = Qux2::Floof(42, String::from("foo"));
    let delta = val0.delta(&val1)?;
    let expected = Qux2Delta::Floof(std::marker::PhantomData, None);
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn enum_tuple_struct_variant__apply__same_values() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Floof(42, String::from("foo"));
    let delta = Qux2Delta::Floof(std::marker::PhantomData, None);
    let val1 = val0.apply(delta)?;
    let expected: Qux2<String, ()> = Qux2::Floof(42, String::from("foo"));
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}

#[test]
pub fn enum_tuple_struct_variant__delta__different_values() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Floof(42, String::from("foo"));
    let val1: Qux2<String, ()> = Qux2::Floof(50, String::from("bar"));
    let delta = val0.delta(&val1)?;
    let expected = Qux2Delta::Floof(
        std::marker::PhantomData,
        Some(StringDelta(Some("bar".into())))
    );
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn enum_tuple_struct_variant__apply__different_values() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Floof(42, String::from("foo"));
    let delta = Qux2Delta::Floof(
        std::marker::PhantomData,
        Some(StringDelta(Some("bar".into())))
    );
    let val1 = val0.apply(delta)?;
    let expected: Qux2<String, ()> = Qux2::Floof(42, String::from("bar"));
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}




#[test]
pub fn enum_unit_struct_variant__delta() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Gah;
    let val1: Qux2<String, ()> = Qux2::Gah;
    let delta = val0.delta(&val1)?;
    let expected = Qux2Delta::Gah;
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn enum_unit_struct_variant__apply() -> DeltaResult<()> {
    let val0: Qux2<String, ()> = Qux2::Gah;
    let delta = Qux2Delta::Gah;
    let val1 = val0.apply(delta)?;
    let expected: Qux2<String, ()> = Qux2::Gah;
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}




#[test]
pub fn struct__delta__same_values() -> DeltaResult<()> {
    let val0: Foo0<u16> = Foo0 {
        f0: (),
        f1: 42 as u16,
        f2: "hello world".into()
    };
    let val1: Foo0<u16> = Foo0 {
        f0: (),
        f1: 300,
        f2: "hello world!!!".into()
    };
    let delta: Foo0Delta<u16> = val0.delta(&val1)?;
    let expected: Foo0Delta<u16> = Foo0Delta {
        f0: std::marker::PhantomData,
        f1: Some(300u16.into_delta()?),
        f2: Some("hello world!!!".to_string().into_delta()?),
    };
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn struct__apply__same_values() -> DeltaResult<()> {
    let val0: Foo0<u16> = Foo0 {
        f0: (),
        f1: 42 as u16,
        f2: "hello world".into()
    };
    let delta: Foo0Delta<u16> = Foo0Delta {
        f0: std::marker::PhantomData,
        f1: Some(300u16.into_delta()?),
        f2: Some("hello world!!!".to_string().into_delta()?),
    };
    let val1 = val0.apply(delta)?;
    let expected: Foo0<u16> = Foo0 {
        f0: (),
        f1: 300,
        f2: String::from("hello world!!!")
    };
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}

#[test]
pub fn struct__delta__different_values() -> DeltaResult<()> {
    let val0: Foo0<u16> = Foo0 {
        f0: (),
        f1: 42 as u16,
        f2: "hello world".into()
    };
    let val1: Foo0<u16> = Foo0 {
        f0: (),
        f1: 300,
        f2: "hai world".into()
    };
    let delta: Foo0Delta<u16> = val0.delta(&val1)?;
    let expected: Foo0Delta<u16> = Foo0Delta {
        f0: std::marker::PhantomData,
        f1: Some(300u16.into_delta()?),
        f2: Some("hai world".to_string().into_delta()?),
    };
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn struct__apply__different_values() -> DeltaResult<()> {
    let val0: Foo0<u16> = Foo0 {
        f0: (),
        f1: 42 as u16,
        f2: "hello world".into()
    };
    let delta: Foo0Delta<u16> = Foo0Delta {
        f0: std::marker::PhantomData,
        f1: Some(300u16.into_delta()?),
        f2: Some("hai world".to_string().into_delta()?),
    };
    let val1 = val0.apply(delta)?;
    let expected: Foo0<u16> = Foo0 {
        f0: (),
        f1: 300,
        f2: "hai world".into()
    };
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}




#[test]
pub fn tuple_struct__delta__same_values() -> DeltaResult<()> {
    let val0: Bar<u16> = Bar(42u8, 70u16);
    let val1: Bar<u16> = Bar(42u8, 70u16);
    let delta: BarDelta<u16> = val0.delta(&val1)?;
    let expected: BarDelta<u16> = BarDelta(None, None);
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn tuple_struct__apply__same_values() -> DeltaResult<()>  {
    let val0: Bar<u16> = Bar(42u8, 70u16);
    let delta: BarDelta<u16> = BarDelta(
        Some(100u8.into_delta()?),
        Some(300u16.into_delta()?),
    );
    let val1: Bar<u16> = val0.apply(delta)?;
    let expected: Bar<u16> = Bar(100u8, 300u16);
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}

#[test]
pub fn tuple_struct__delta__different_values() -> DeltaResult<()> {
    let val0: Bar<u16> = Bar( 42u8,  70u16);
    let val1: Bar<u16> = Bar(100u8, 300u16);
    let delta: BarDelta<u16> = val0.delta(&val1)?;
    let expected: BarDelta<u16> = BarDelta(
        Some(100u8.into_delta()?),
        Some(300u16.into_delta()?),
    );
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn tuple_struct__apply__different_values() -> DeltaResult<()>  {
    let val0: Bar<u16> = Bar(42u8,  70u16);
    let delta: BarDelta<u16> = BarDelta(
        Some(100u8.into_delta()?),
        Some(300u16.into_delta()?),
    );
    let val1: Bar<u16> = val0.apply(delta)?;
    let expected: Bar<u16> = Bar(100u8, 300u16);
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}




#[test]
pub fn unit_struct__delta() -> DeltaResult<()> {
    let val0 = Baz;
    let val1 = Baz;
    let delta: BazDelta = val0.delta(&val1)?;
    let expected = BazDelta;
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn unit_struct__apply() -> DeltaResult<()>  {
    let val0 = Baz;
    let delta = BazDelta;
    let val1: Baz = val0.apply(delta)?;
    let expected = Baz;
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}




#[test]
pub fn nested_data__delta() -> DeltaResult<()> {
    let val0: Corge<Corge<(), bool>, ()> = Corge::Grault(
        42u8,
        Corge::Floof { one: 100u8, two: (), three: true }
    );
    let val1: Corge<Corge<(), bool>, ()> = Corge::Grault(
        40u8,
        Corge::Floof { one: 72u8, two: (), three: true }
    );
    let delta = val0.delta(&val1)?;
    let expected: CorgeDelta<Corge<(), bool>, ()> = CorgeDelta::Grault(
        Some(U8Delta(Some(40u8))),
        Some(CorgeDelta::Floof {
            one:   Some(U8Delta(Some(72u8))),
            two:   None,
            three: None
        })
    );
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn nested_data__apply() -> DeltaResult<()>  {
    let val0: Corge<Corge<(), bool>, ()> = Corge::Grault(
        42u8,
        Corge::Floof { one: 100u8, two: (), three: true }
    );
    let delta: CorgeDelta<_, _> = CorgeDelta::Grault(
        Some(U8Delta(Some(40u8))),
        Some(CorgeDelta::Floof {
            one: Some(U8Delta(Some(72u8))),
            two: None,
            three: None,
        })
    );
    let val1 = val0.apply(delta)?;
    let expected: Corge<Corge<(), bool>, ()> = Corge::Grault(
        40u8,
        Corge::Floof { one: 72u8, two: (), three: true }
    );
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}
