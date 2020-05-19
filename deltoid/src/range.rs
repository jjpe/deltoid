//!

use crate::{Deltoid, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de;
use serde::ser::SerializeMap;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Range;


impl<T> Deltoid for Range<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
    + Serialize
    + for<'de> Deserialize<'de>
    + IntoDelta
    + FromDelta
{
    type Delta = RangeDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        match &delta.0 {
            Some(range) => Ok(range.start.clone() .. range.end.clone()),
            None        => Ok(self.start.clone() ..  self.end.clone()),
        }
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(RangeDelta(if self == rhs {
            None
        } else {
            Some(rhs.clone())
        }))
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
pub struct RangeDelta<T>(#[doc(hidden)] pub Option<Range<T>>);

impl<T> IntoDelta for Range<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        Ok(RangeDelta(Some(self)))
    }
}

impl<T> FromDelta for Range<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        Ok(delta.0.ok_or_else(|| ExpectedValue!("RangeDelta<K, V>"))?)
    }
}



impl<T: Deltoid + Clone + Serialize> Serialize for RangeDelta<T> {
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
where T: Deltoid + Clone +  Deserialize<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {

        struct DeltaVisitor<T2> {
            _phantom: PhantomData<T2>,
        }

        impl<'de, T2> de::Visitor<'de> for DeltaVisitor<T2>
        where T2: Deltoid + Clone + Deserialize<'de> {
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
    fn calculate_delta_for_Range__same_values() -> DeltaResult<()> {
        let range0 = 1..10;
        let range1 = 1..10;
        let delta: <Range<usize> as Deltoid>::Delta = range0.delta(&range1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{}");
        let delta1: <Range<usize> as Deltoid>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn calculate_delta_for_Range__different_values() -> DeltaResult<()> {
        let range0 = 1..10;
        let range1 = 1..11;
        let delta: <Range<usize> as Deltoid>::Delta = range0.delta(&range1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"0\":{\"start\":1,\"end\":11}}");
        let delta1: <Range<usize> as Deltoid>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn apply_delta_for_Range_same_values() -> DeltaResult<()> {
        let range0 = 1..10;
        let range1 = 1..10;
        let delta: <Range<usize> as Deltoid>::Delta = range0.delta(&range1)?;
        let range2 = range0.apply_delta(&delta)?;
        assert_eq!(range0, range2);
        Ok(())
    }

    #[test]
    fn apply_delta_for_Range_different_values() -> DeltaResult<()> {
        let range0 = 1..10;
        let range1 = 1..11;
        let delta: <Range<usize> as Deltoid>::Delta = range0.delta(&range1)?;
        let range2 = range0.apply_delta(&delta)?;
        assert_eq!(range1, range2);
        Ok(())
    }
}
