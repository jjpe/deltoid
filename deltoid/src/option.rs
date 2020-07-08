//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use std::fmt::Debug;
use serde::{Deserialize, Serialize};

impl<T> Core for Option<T>
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = OptionDelta<T>;
}

impl<T> Apply for Option<T>
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        Ok(match (&self, delta) {
            (None,    Self::Delta::None)    => None,
            (Some(_), Self::Delta::None)    => self.clone(),
            (None,    Self::Delta::Some(ref d)) => Some(
                <T>::from_delta(d.clone(/*TODO: rm clone for more efficiency*/))?
            ),
            (Some(t), Self::Delta::Some(ref d)) =>
                Some(t.apply(d.clone(/*TODO: rm clone for more efficiency*/))?),
        })
    }
}

impl<T> Delta for Option<T>
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(match (self, rhs) {
            (Some(lhs), Some(rhs)) => Self::Delta::Some(lhs.delta(&rhs)?),
            (None,      Some(rhs)) => Self::Delta::Some(rhs.clone().into_delta()?),
            (Some(_),   None)      => Self::Delta::None,
            (None,      None)      => Self::Delta::None,
        })
    }
}

impl<T> FromDelta for Option<T>
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        Ok(match delta {
            Self::Delta::None => None,
            Self::Delta::Some(delta) => Some(<T>::from_delta(delta)?),
        })
    }
}

impl<T> IntoDelta for Option<T>
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        Ok(match self {
            Self::None => OptionDelta::None,
            Self::Some(t) => OptionDelta::Some(t.into_delta()?),
        })
    }
}



#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum OptionDelta<T: Core> {
    None,
    Some(<T as Core>::Delta),
}


#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn Option__delta__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let option0 = Some(foo);
        let option1 = Some(bar);
        let delta: <Option<String> as Core>::Delta = option0.delta(&option1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"Some\":\"foo\"}");
        let delta1: <Option<String> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Option__delta__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let option0 = Some(foo);
        let option1 = Some(bar);
        let delta: <Option<String> as Core>::Delta = option0.delta(&option1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "{\"Some\":\"bar\"}");
        let delta1: <Option<String> as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Option__apply__same_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("foo");
        let option0 = Some(foo);
        let option1 = Some(bar);
        let delta: <Option<String> as Core>::Delta = option0.delta(&option1)?;
        let option2 = option0.apply(delta)?;
        assert_eq!(option1, option2);
        Ok(())
    }

    #[test]
    fn Option__apply__different_values() -> DeltaResult<()> {
        let foo = String::from("foo");
        let bar = String::from("bar");
        let option0 = Some(foo);
        let option1 = Some(bar);
        let delta: <Option<String> as Core>::Delta = option0.delta(&option1)?;
        let option2 = option0.apply(delta)?;
        assert_eq!(option1, option2);
        Ok(())
    }
}
