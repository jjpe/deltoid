//!

use crate::{Deltoid, DeltaError, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};

impl<T, E> Deltoid for Result<T, E>
where T: Deltoid + FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize,
      E: Deltoid + FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize {
    type Delta = ResultDelta<T, E>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        match (self, delta) {
            (Result::Ok(ok), ResultDelta::None) => Ok(Ok(ok.clone())),
            (Result::Ok(ok), ResultDelta::OkDelta(delta)) => {
                Ok(Ok(ok.apply_delta(delta)?))
            },
            (Result::Ok(_ok), delta @ ResultDelta::ErrDelta(_)) => {
                Ok(Self::from_delta(delta.clone())?)
            },
            (Result::Err(err), ResultDelta::None) => Ok(Err(err.clone())),
            (Result::Err(_err), delta @ ResultDelta::OkDelta(_)) => {
                Ok(Self::from_delta(delta.clone())?)
            },
            (Result::Err(err), ResultDelta::ErrDelta(delta)) => {
                Ok(Err(err.apply_delta(delta)?))
            },
        }
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        match (self, rhs) {
            (Ok(lhs), Ok(rhs)) if lhs == rhs => Ok(ResultDelta::None),
            (Ok(lhs), Ok(rhs)) =>
                Ok(ResultDelta::OkDelta(lhs.delta(rhs)?)),
            (Ok(_lhs), Err(rhs)) =>
                Ok(ResultDelta::ErrDelta(rhs.clone().into_delta()?)),
            (Err(_lhs), Ok(rhs)) =>
                Ok(ResultDelta::OkDelta(rhs.clone().into_delta()?)),
            (Err(lhs), Err(rhs)) if lhs == rhs => Ok(ResultDelta::None),
            (Err(_lhs), Err(rhs)) =>
                Ok(ResultDelta::ErrDelta(rhs.clone().into_delta()?)),
        }
    }
}

#[derive(
    Clone, Debug, PartialEq,
    serde_derive::Deserialize, serde_derive::Serialize
)]
pub enum ResultDelta<T, E>
where T: Deltoid,
      E: Deltoid {
    OkDelta(<T as Deltoid>::Delta),
    ErrDelta(<E as Deltoid>::Delta),
    None
}

impl<T, E> IntoDelta for Result<T, E>
where T: FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize,
      E: FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize {
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        match self {
            Ok(ok)   => Ok(ResultDelta::OkDelta(ok.into_delta()?)),
            Err(err) => Ok(ResultDelta::ErrDelta(err.into_delta()?)),
        }
    }
}

impl<T, E> FromDelta for Result<T, E>
where T: FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize,
      E: FromDelta + IntoDelta + for<'de> Deserialize<'de> + Serialize {
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        match delta {
            ResultDelta::None => Err(DeltaError::ExpectedValue),
            ResultDelta::OkDelta(delta) =>
                Ok(Self::Ok(<T>::from_delta(delta)?)),
            ResultDelta::ErrDelta(delta) =>
                Ok(Self::Err(<E>::from_delta(delta)?)),
        }
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
        assert_eq!(json_string, "\"None\"");
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
        assert_eq!(json_string, "{\"OkDelta\":\"bar\"}");
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
        assert_eq!(json_string, "\"None\"");
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
        assert_eq!(json_string, "{\"ErrDelta\":\"bar\"}");
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
