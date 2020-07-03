//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de;
use serde::ser::SerializeMap;
use std::borrow::{Borrow, Cow, ToOwned};
use std::fmt::{self, Debug};
use std::marker::PhantomData;


impl<'a, B> Core for Cow<'a, B>
where B: Clone + Debug + PartialEq + Core + ToOwned
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = CowDelta<'a, B>;
}

impl<'a, B> Apply for Cow<'a, B>
where B: Apply + FromDelta + ToOwned
    + for<'de> Deserialize<'de>
    + Serialize,
      <B as ToOwned>::Owned: Debug
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let lhs: &B = self.borrow();
        Ok(if let Some(delta) = delta.inner {
            Cow::Owned(lhs.apply(delta)?.to_owned())
        } else {
            self.clone()
        })
    }
}

impl<'a, B> Delta for Cow<'a, B>
where B: Delta + ToOwned
    + for<'de> Deserialize<'de>
    + Serialize,
      <B as ToOwned>::Owned: Debug
{
    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta> {
        let (lhs, rhs): (&B, &B) = (self.borrow(), other.borrow());
        Ok(CowDelta {
            inner: if self == other {
                None
            } else {
                Some(lhs.delta(rhs)?)
            },
            _phantom: PhantomData,
        })
    }
}

impl<'a, B> FromDelta for Cow<'a, B>
where B: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        let delta: B::Delta = delta.inner
            .ok_or_else(|| ExpectedValue!("CowDelta<'a, B>"))?;
        Ok(Cow::Owned(<B>::from_delta(delta)?.to_owned()))
    }
}

impl<'a, B> IntoDelta for Cow<'a, B>
where B: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        Ok(CowDelta {
            inner: Some((self.borrow() as &B).clone().into_delta()?),
            _phantom: PhantomData,
        })
    }
}



#[derive(Clone, Debug, PartialEq)]
// #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct CowDelta<'a, B: Core> {
    #[doc(hidden)] pub inner: Option<B::Delta>,
    #[doc(hidden)] pub _phantom: PhantomData<&'a B>
}

impl<'a, B> Serialize for CowDelta<'a, B>
where B: Core + for<'de> Deserialize<'de> + Serialize {
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

impl<'de, 'a, B> Deserialize<'de> for CowDelta<'a, B>
where B: Core + Deserialize<'de> + Serialize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct DeltaVisitor<'a2, B2>(PhantomData<&'a2 B2>);

        impl<'de, 'a2, B2> de::Visitor<'de> for DeltaVisitor<'a2, B2>
        where B2: Core + Deserialize<'de> + Serialize {
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

        deserializer.deserialize_map(DeltaVisitor(PhantomData))
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
        let delta: <Cow<String> as Core>::Delta = cow.delta(&cow2)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: {}", json_string);
        assert_eq!(json_string, "{}");
        let delta1: <Cow<String> as Core>::Delta = serde_json::from_str(
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
        let delta: <Cow<String> as Core>::Delta = cow.delta(&cow2)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: {}", json_string);
        assert_eq!(json_string, "{\"inner\":\"bar\"}");
        let delta1: <Cow<String> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }
}
