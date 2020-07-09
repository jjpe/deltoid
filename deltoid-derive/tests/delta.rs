//!
#![allow(non_snake_case)]

#[allow(unused)] use deltoid::{
    Core, Apply, Delta, DeltaResult, FromDelta, IntoDelta
};
use deltoid_derive::Delta;


#[derive(
    Clone, Debug, PartialEq, Delta,
    serde_derive::Deserialize, serde_derive::Serialize
)]
pub enum Qux1<T: Default> {
    Blah { #[delta(ignore_field)] one: u8, two: () },
    Floof(u8, T),
    Flah { one: Box<Qux2<(), ()>> },
    Gah,
}

#[derive(
    Debug, PartialEq, Clone, Delta,
    serde_derive::Deserialize, serde_derive::Serialize
)]
pub enum Qux2<T, U: Default> {
    Floof(#[delta(ignore_field)] u8, T),
    Blah { one: u8, #[delta(ignore_field)] two: U },
    Flah { one: Box<Qux1<()>> },
    Gah,
}

#[allow(unused)]
#[derive(Clone, Debug, PartialEq)]
#[derive(Delta)]
enum Corge<Tx, U: Copy> {
    Quux,
    Grault(
        #[delta(ignore_field)]
        u8,
        Tx
    ),
    Floof {
        #[delta(ignore_field)]
        one: u8,
        two: Tx,
        three: U
    },
}

#[derive(Clone, Debug, PartialEq, Delta)]
pub struct Foo0<F: Copy> where F: Copy {
    #[delta(ignore_field)]
    f0: (),
    f1: F,
    // #[delta(ignore_field)]
    f2: String,
}

#[derive(Clone, Debug, PartialEq, Delta)]
pub struct Bar<S: Copy>(u8, #[delta(ignore_field)] S)
where S: std::fmt::Debug + Default;

#[derive(Clone, Debug, PartialEq)]
#[derive(Delta)]
pub struct Baz;

#[derive(Clone, Debug, PartialEq)]
#[derive(Delta)]
pub struct Plow(std::borrow::Cow<'static, String>);






#[test]
pub fn struct__delta() -> DeltaResult<()> {
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
pub fn struct__apply() -> DeltaResult<()>  {
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
pub fn tuple_struct__delta() -> DeltaResult<()> {
    let val0: Bar<u16> = Bar(42u8, 300u16);
    let val1: Bar<u16> = Bar(100u8, 300u16);
    let delta: BarDelta<u16> = val0.delta(&val1)?;
    let expected: BarDelta<u16> = BarDelta(
        Some(100u8.into_delta()?),
        std::marker::PhantomData
    );
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn tuple_struct__apply() -> DeltaResult<()>  {
    let val0: Bar<u16> = Bar(42u8, 300u16);
    let delta: BarDelta<u16> = BarDelta(
        Some(100u8.into_delta()?),
        std::marker::PhantomData
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
    let expected: BazDelta = BazDelta;
    assert_eq!(delta, expected, "{:#?} != {:#?}", delta, expected);
    Ok(())
}

#[test]
pub fn unit_struct__apply() -> DeltaResult<()>  {
    let val0 = Baz;
    let delta: BazDelta = BazDelta;
    let val1: Baz = val0.apply(delta)?;
    let expected = Baz;
    assert_eq!(val1, expected, "{:#?} != {:#?}", val1, expected);
    Ok(())
}
