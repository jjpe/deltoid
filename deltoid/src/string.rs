//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};

impl Core for String {
    type Delta = StringDelta;
}

impl Apply for String {
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        Self::from_delta(delta)
    }
}

impl Delta for String {
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        rhs.clone().into_delta()
    }
}

impl FromDelta for String {
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        delta.0.ok_or_else(|| ExpectedValue!("StringDelta<T>"))
    }
}

impl IntoDelta for String {
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        Ok(StringDelta(Some(self)))
    }
}


#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct StringDelta( // TODO: Improve delta space efficiency
    #[doc(hidden)] pub Option<String>
);


#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn String__delta__same_values() -> DeltaResult<()> {
        let s0 = String::from("foo");
        let s1 = String::from("foo");
        let delta: <String as Core>::Delta = s0.delta(&s1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: {}", json_string);
        assert_eq!(json_string, "\"foo\"");
        let delta1: <String as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        assert_eq!(delta, String::from("foo").into_delta()?);
        Ok(())
    }

    #[test]
    fn String__delta__different_values() -> DeltaResult<()> {
        let s0 = String::from("foo");
        let s1 = String::from("bar");
        let delta: <String as Core>::Delta = s0.delta(&s1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: {}", json_string);
        assert_eq!(json_string, "\"bar\"");
        let delta1: <String as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        assert_eq!(delta, String::from("bar").into_delta()?);
        Ok(())
    }

    #[test]
    fn String__apply__same_values() -> DeltaResult<()> {
        let s0 = String::from("foo");
        let s1 = String::from("foo");
        let delta: <String as Core>::Delta = s0.delta(&s1)?;
        let s2 = s0.apply(delta)?;
        assert_eq!(s1, s2);
        Ok(())
    }

    #[test]
    fn String__apply__different_values() -> DeltaResult<()> {
        let s0 = String::from("foo");
        let s1 = String::from("bar");
        let delta: <String as Core>::Delta = s0.delta(&s1)?;
        let s2 = s0.apply(delta)?;
        assert_eq!(s1, s2);
        Ok(())
    }
}
