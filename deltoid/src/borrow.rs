//!

use crate::{DeltaError, DeltaResult, Deltoid};
use crate::convert::{FromDelta, IntoDelta};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de;
use serde::ser::SerializeMap;
use std::borrow::{Borrow, Cow, ToOwned};
use std::fmt;
use std::marker::PhantomData;


impl<'a, B> Deltoid for Cow<'a, B>
where B: Clone + std::fmt::Debug + PartialEq + Deltoid + ToOwned
        + Serialize
        + for<'de> Deserialize<'de>,
      <B as ToOwned>::Owned: std::fmt::Debug
{
    type Delta = CowDelta<'a, B>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &B = self.borrow();
        if let Some(delta) = delta.inner.as_ref() {
            lhs.apply_delta(delta)
                .map(|new| new.to_owned())
                .map(Cow::Owned)
        } else {
            Ok(self.clone())
        }
    }

    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta> {
        let (lhs, rhs): (&B, &B) = (self.borrow(), other.borrow());
        Ok(CowDelta {
            inner: if self != other {
                Some(lhs.delta(rhs)?)
            } else {
                None
            },
            _phantom: PhantomData,
        })
    }
}



impl<'a, B> IntoDelta for Cow<'a, B>
where B: IntoDelta + Serialize + for<'de> Deserialize<'de> {
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        Ok(CowDelta {
            inner: Some((self.borrow() as &B).clone().into_delta()?),
            _phantom: PhantomData,
        })
    }
}

impl<'a, B> FromDelta for Cow<'a, B>
where B: FromDelta + Serialize + for<'de> Deserialize<'de> {
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        let delta: <B as Deltoid>::Delta = delta.inner
            .ok_or(DeltaError::ExpectedValue)?;
        B::from_delta(delta)
            .map(|b: B| b.to_owned())
            .map(Cow::Owned)
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct CowDelta<'a, B: Deltoid + Clone> {
    #[doc(hidden)] pub inner: Option<<B as Deltoid>::Delta>,
    #[doc(hidden)] pub _phantom: PhantomData<&'a B>
}

impl<'a, B: Deltoid + Clone> Serialize for CowDelta<'a, B> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: Serializer {
        let mut num_fields = 0;
        if self.inner.is_some() { num_fields += 1; }
        let mut s = serializer.serialize_map(Some(num_fields))?;
        if let Some(inner) = &self.inner {
            s.serialize_entry("inner", inner)?;
        }
        s.end()
    }
}

impl<'de, 'a, B: Deltoid + Clone> Deserialize<'de> for CowDelta<'a, B> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {

        struct DeltaVisitor<'a2, B2> {
            _phantom: PhantomData<&'a2 B2>,
        }

        impl<'de, 'a2, B2> de::Visitor<'de> for DeltaVisitor<'a2, B2>
        where B2: Deltoid + Clone {
            type Value = CowDelta<'a2, B2>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a CowDelta")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where M: de::MapAccess<'de> {
                let mut delta: Self::Value = Self::Value {
                    inner: None,
                    _phantom: PhantomData,
                };
                const EXPECTED_FIELDS: &[&str] = &["inner"];
                while let Some((key, value)) = map.next_entry()? {
                    match (key, value) {
                        ("inner", value) =>  delta.inner = Some(value),
                        (field_name, _) => return Err(de::Error::unknown_field(
                            field_name, EXPECTED_FIELDS
                        ))?,
                    }
                }
                Ok(delta)
            }
        }

        deserializer.deserialize_map(DeltaVisitor {
            _phantom: PhantomData
        })
    }
}


#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn calculate_delta_for_Cow__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let cow:  Cow<String> = Cow::Borrowed(&foo);
        let cow2: Cow<String> = Cow::Borrowed(&bar);
        let delta: <Cow<String> as Deltoid>::Delta = cow.delta(&cow2)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{}");
        let delta1: <Cow<String> as Deltoid>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn calculate_delta_for_Cow__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let cow:  Cow<String> = Cow::Borrowed(&foo);
        let cow2: Cow<String> = Cow::Borrowed(&bar);
        let delta: <Cow<String> as Deltoid>::Delta = cow.delta(&cow2)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"inner\":\"bar\"}");
        let delta1: <Cow<String> as Deltoid>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }
}
