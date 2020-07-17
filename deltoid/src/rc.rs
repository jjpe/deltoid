//! A Deltoid impl for [`Rc`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de;
use serde::ser::SerializeMap;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::rc::Rc;


impl<T> Core for Rc<T>
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = RcDelta<T>;
}

impl<T> Apply for Rc<T>
where T: Apply
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let lhs: &T = self.as_ref();
        match delta.0 {
            None => Ok(self.clone()),
            Some(delta) => lhs.apply(*delta).map(Rc::new),
        }
    }
}

impl<T> Delta for Rc<T>
where T: Delta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lhs: &T = self.as_ref();
        let rhs: &T = rhs.as_ref();
        Ok(RcDelta(if lhs == rhs {
            None
        } else {
            Some(Box::new(lhs.delta(rhs)?))
        }))
    }
}

impl<T> FromDelta for Rc<T>
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let delta = delta.0.ok_or_else(|| ExpectedValue!("RcDelta<T>"))?;
        <T>::from_delta(*delta).map(Rc::new)
    }
}

impl<T> IntoDelta for Rc<T>
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let thing: T = self.as_ref().clone();
        thing.into_delta().map(Box::new).map(Some).map(RcDelta)
    }
}



#[derive(Clone, PartialEq)]
pub struct RcDelta<T: Core>(
    #[doc(hidden)] pub Option<Box<<T as Core>::Delta>>
);

impl<T: Core> std::fmt::Debug for RcDelta<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self.0 {
            Some(d) => write!(f, "RcDelta({:#?})", d),
            None    => write!(f, "RcDelta(None)"),
        }
    }
}

impl<T: Core> Serialize for RcDelta<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut num_fields = 0;
        if self.0.is_some() { num_fields += 1; }
        let mut s = serializer.serialize_map(Some(num_fields))?;
        if let Some(inner) = &self.0 {
            s.serialize_entry("0", inner)?;
        }
        s.end()
    }
}

impl<'de, T: Core + Clone> Deserialize<'de> for RcDelta<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct DeltaVisitor<T2>(PhantomData<T2>);

        impl<'de, T2> de::Visitor<'de> for DeltaVisitor<T2>
        where T2: Core + Clone {
            type Value = RcDelta<T2>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a RcDelta")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where M: de::MapAccess<'de> {
                let mut delta: Self::Value = RcDelta(None);
                const EXPECTED_FIELDS: &[&str] = &["0"];
                while let Some((key, value)) = map.next_entry()? {
                    match (key, value) {
                        ("0", value) =>  delta.0 = Some(value),
                        (field_name, _) => return Err(de::Error::unknown_field(
                            field_name, EXPECTED_FIELDS
                        ))?,
                    }
                }
                Ok(delta)
            }
        }

        deserializer.deserialize_map(DeltaVisitor(PhantomData))
    }
}



#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn Rc__delta___same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Rc::new(foo);
        let box1 = Rc::new(bar);
        let delta: <Rc<String> as Core>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{}");
        let delta1: <Rc<String> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Rc__delta___different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Rc::new(foo);
        let box1 = Rc::new(bar);
        let delta: <Rc<String> as Core>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"0\":\"bar\"}");
        let delta1: <Rc<String> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Rc__apply__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Rc::new(foo);
        let box1 = Rc::new(bar);
        let delta: <Rc<String> as Core>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply(delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }

    #[test]
    fn Rc__apply__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Rc::new(foo);
        let box1 = Rc::new(bar);
        let delta: <Rc<String> as Core>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply(delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }
}
