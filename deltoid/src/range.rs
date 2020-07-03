//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de;
use serde::ser::SerializeMap;
use std::fmt::{self, Debug};
use std::marker::PhantomData;
use std::ops::Range;


impl<T> Core for Range<T>
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = RangeDelta<T>;
}

impl<T> Apply for Range<T>
where T: Apply
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        match delta.0 {
            Some(range) => Ok(range.start .. range.end),
            None        => Ok(self.start.clone() ..  self.end.clone()),
        }
    }
}

impl<T> Delta for Range<T>
where T: Delta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(RangeDelta(if self == rhs {
            None
        } else {
            Some(rhs.clone())
        }))
    }
}

impl<T> FromDelta for Range<T>
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        Ok(delta.0.ok_or_else(|| ExpectedValue!("RangeDelta<K, V>"))?)
    }
}

impl<T> IntoDelta for Range<T>
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        Ok(RangeDelta(Some(self)))
    }
}



#[derive(Clone, Debug, PartialEq, Hash)]
pub struct RangeDelta<T>(#[doc(hidden)] pub Option<Range<T>>);

impl<T> Serialize for RangeDelta<T>
where T: Core
    + Clone
    + Serialize
{
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

impl<'de, T> Deserialize<'de> for RangeDelta<T>
where T: Core
    + Clone
    + Deserialize<'de>
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct DeltaVisitor<T2>(PhantomData<T2>);

        impl<'de, T2> de::Visitor<'de> for DeltaVisitor<T2>
        where T2: Core
            + Clone
            + Deserialize<'de>
        {
            type Value = RangeDelta<T2>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a RangeDelta")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where M: de::MapAccess<'de> {
                let mut delta: Self::Value = RangeDelta(None);
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
    fn calculate_delta_for_Range__same_values() -> DeltaResult<()> {
        let range0 = 1..10;
        let range1 = 1..10;
        let delta: <Range<usize> as Core>::Delta = range0.delta(&range1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{}");
        let delta1: <Range<usize> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn calculate_delta_for_Range__different_values() -> DeltaResult<()> {
        let range0 = 1..10;
        let range1 = 1..11;
        let delta: <Range<usize> as Core>::Delta = range0.delta(&range1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"0\":{\"start\":1,\"end\":11}}");
        let delta1: <Range<usize> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn apply_delta_for_Range_same_values() -> DeltaResult<()> {
        let range0 = 1..10;
        let range1 = 1..10;
        let delta: <Range<usize> as Core>::Delta = range0.delta(&range1)?;
        let range2 = range0.apply(delta)?;
        assert_eq!(range0, range2);
        Ok(())
    }

    #[test]
    fn apply_delta_for_Range_different_values() -> DeltaResult<()> {
        let range0 = 1..10;
        let range1 = 1..11;
        let delta: <Range<usize> as Core>::Delta = range0.delta(&range1)?;
        let range2 = range0.apply(delta)?;
        assert_eq!(range1, range2);
        Ok(())
    }
}
