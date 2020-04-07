//!

use crate::{Deltoid, DeltaError, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};

impl<T, E> Deltoid for Result<T, E>
where T: Deltoid + FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize,
      E: Deltoid + FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize {
    type Delta = ResultDelta<T, E>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        Self::from_delta(delta.clone())
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        rhs.clone().into_delta()
    }
}

#[derive(
    Clone, Debug, PartialEq,
    serde_derive::Deserialize, serde_derive::Serialize
)]
pub struct ResultDelta<T, E>(#[doc(hidden)]pub Option<Result<T, E>>);

impl<T, E> IntoDelta for Result<T, E>
where T: FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize,
      E: FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize {
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        Ok(ResultDelta(Some(self)))
    }
}

impl<T, E> FromDelta for Result<T, E>
where T: FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize,
      E: FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize {
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        delta.0.ok_or(DeltaError::ExpectedValue)
    }
}



#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn calculate_delta_for_Ok_Result__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Ok(foo);
        let box1 = Ok(bar);
        let delta: <Result<String, ()> as Deltoid>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"Ok\":\"foo\"}");
        let delta1: <Result<String, ()> as Deltoid>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn calculate_delta_for_Ok_Result__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Ok(foo);
        let box1 = Ok(bar);
        let delta: <Result<String, ()> as Deltoid>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"Ok\":\"bar\"}");
        let delta1: <Result<String, ()> as Deltoid>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn apply_delta_for_Ok_Result_same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Ok(foo);
        let box1 = Ok(bar);
        let delta: <Result<String, ()> as Deltoid>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply_delta(&delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }

    #[test]
    fn apply_delta_for_Ok_Result_different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Ok(foo);
        let box1 = Ok(bar);
        let delta: <Result<String, ()> as Deltoid>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply_delta(&delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }





    #[test]
    fn calculate_delta_for_Err_Result__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Err(foo);
        let box1 = Err(bar);
        let delta: <Result<(), String> as Deltoid>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"Err\":\"foo\"}");
        let delta1: <Result<(), String> as Deltoid>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn calculate_delta_for_Err_Result__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Err(foo);
        let box1 = Err(bar);
        let delta: <Result<(), String> as Deltoid>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"Err\":\"bar\"}");
        let delta1: <Result<(), String> as Deltoid>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn apply_delta_for_Err_Result_same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Err(foo);
        let box1 = Err(bar);
        let delta: <Result<(), String> as Deltoid>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply_delta(&delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }

    #[test]
    fn apply_delta_for_Err_Result_different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Err(foo);
        let box1 = Err(bar);
        let delta: <Result<(), String> as Deltoid>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply_delta(&delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }
}
