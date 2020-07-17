//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

impl<T, E> Core for Result<T, E>
where T: Clone + Debug + PartialEq + Core + for<'de> Deserialize<'de> + Serialize,
      E: Clone + Debug + PartialEq + Core + for<'de> Deserialize<'de> + Serialize,
{
    type Delta = ResultDelta<T, E>;
}

impl<T, E> Apply for Result<T, E>
where T: Apply + FromDelta + for<'de> Deserialize<'de> + Serialize,
      E: Apply + FromDelta + for<'de> Deserialize<'de> + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        match (self, &delta/*TODO: match by value*/) {
            (Result::Ok(ok), ResultDelta::None) => Ok(Ok(ok.clone())),
            (Result::Ok(ok), ResultDelta::OkDelta(delta)) => {
                Ok(Ok(ok.apply(delta.clone(/*TODO: rm*/))?))
            },
            (Result::Ok(_ok), delta @ ResultDelta::ErrDelta(_)) => {
                Ok(Self::from_delta(delta.clone())?)
            },
            (Result::Err(err), ResultDelta::None) => Ok(Err(err.clone())),
            (Result::Err(_err), delta @ ResultDelta::OkDelta(_)) => {
                Ok(Self::from_delta(delta.clone())?)
            },
            (Result::Err(err), ResultDelta::ErrDelta(delta)) => {
                Ok(Err(err.apply(delta.clone(/*TODO: rm*/))?))
            },
        }
    }
}

impl<T, E> Delta for Result<T, E>
where T: Delta + IntoDelta + for<'de> Deserialize<'de> + Serialize,
      E: Delta + IntoDelta + for<'de> Deserialize<'de> + Serialize
{
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

impl<T, E> FromDelta for Result<T, E>
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
      E: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        match delta {
            ResultDelta::None => Err(ExpectedValue!("ResultDelta<T, E>")),
            ResultDelta::OkDelta(delta) =>
                Ok(Self::Ok(<T>::from_delta(delta)?)),
            ResultDelta::ErrDelta(delta) =>
                Ok(Self::Err(<E>::from_delta(delta)?)),
        }
    }
}

impl<T, E> IntoDelta for Result<T, E>
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
      E: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        match self {
            Ok(ok)   => Ok(ResultDelta::OkDelta(ok.into_delta()?)),
            Err(err) => Ok(ResultDelta::ErrDelta(err.into_delta()?)),
        }
    }
}



#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum ResultDelta<T: Core, E: Core> {
    OkDelta(<T as Core>::Delta),
    ErrDelta(<E as Core>::Delta),
    None
}

impl<T, E> std::fmt::Debug for ResultDelta<T, E>
where T: Core, E: Core {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self {
            Self::OkDelta(ok)   => write!(f, "ResultDelta::Ok({:#?})",  ok),
            Self::ErrDelta(err) => write!(f, "ResultDelta::Err({:#?})", err),
            Self::None          => write!(f, "ResultDelta::None"),
        }
    }
}


#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn Result_Ok__delta__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Ok(foo);
        let box1 = Ok(bar);
        let delta: <Result<String, ()> as Core>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "\"None\"");
        let delta1: <Result<String, ()> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Result_Ok__delta__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Ok(foo);
        let box1 = Ok(bar);
        let delta: <Result<String, ()> as Core>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"OkDelta\":\"bar\"}");
        let delta1: <Result<String, ()> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Result_Ok__apply__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Ok(foo);
        let box1 = Ok(bar);
        let delta: <Result<String, ()> as Core>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply(delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }

    #[test]
    fn Result_Ok__apply__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Ok(foo);
        let box1 = Ok(bar);
        let delta: <Result<String, ()> as Core>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply(delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }

    #[test]
    fn Result_Err__delta__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Err(foo);
        let box1 = Err(bar);
        let delta: <Result<(), String> as Core>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "\"None\"");
        let delta1: <Result<(), String> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Result_Err__delta__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Err(foo);
        let box1 = Err(bar);
        let delta: <Result<(), String> as Core>::Delta = box0.delta(&box1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"ErrDelta\":\"bar\"}");
        let delta1: <Result<(), String> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Result_Err__apply__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let box0 = Err(foo);
        let box1 = Err(bar);
        let delta: <Result<(), String> as Core>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply(delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }

    #[test]
    fn Result_Err__apply__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let box0 = Err(foo);
        let box1 = Err(bar);
        let delta: <Result<(), String> as Core>::Delta = box0.delta(&box1)?;
        let box2 = box0.apply(delta)?;
        assert_eq!(box1, box2);
        Ok(())
    }
}
