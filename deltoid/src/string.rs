//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use std::borrow::Cow;

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


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct StringDelta( // TODO: Improve delta space efficiency
    #[doc(hidden)] pub Option<String>
);

impl std::fmt::Debug for StringDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self.0 {
            Some(field) => write!(f, "StringDelta({:#?})", field),
            None        => write!(f, "StringDelta(None)"),
        }
    }
}



#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct Str<'s>(pub Cow<'s, str>);

impl<'s> std::ops::Deref for Str<'s> {
    type Target = str;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<'s> std::clone::Clone for Str<'s> {
    fn clone(&self) -> Self { Self(self.0.to_owned()) }
}

impl<'s> From<&'s str> for Str<'s> {
    fn from(s: &'s str) -> Self { Self(Cow::Borrowed(s)) }
}

impl<'s> From<String> for Str<'s> {
    fn from(s: String) -> Self { Self(Cow::Owned(s)) }
}


impl<'s> Core for Str<'s> {
    type Delta = StrDelta;
}

impl<'s> Apply for Str<'s> {
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        Ok(match delta.0 {
            Some(d) => Self(std::borrow::Cow::Owned(d)),
            None => self.clone(),
        })
    }
}

impl<'s> Delta for Str<'s> {
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        rhs.clone().into_delta()
    }
}

impl<'s> FromDelta for Str<'s> {
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        delta.0
            .map(|s| Self(Cow::Owned(s)))
            .ok_or_else(|| ExpectedValue!("StrDelta"))
    }
}

impl<'s> IntoDelta for Str<'s> {
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        match self.0 {
            Cow::Borrowed(b) => Ok(StrDelta(Some(b.to_owned()))),
            Cow::Owned(o)    => Ok(StrDelta(Some(o))),
        }
    }
}


#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct StrDelta( // TODO: Improve delta space efficiency
    #[doc(hidden)] pub Option<String>
);

impl std::fmt::Debug for StrDelta {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self.0 {
            Some(field) => write!(f, "StrDelta({:#?})", field),
            None        => write!(f, "StrDelta(None)"),
        }
    }
}



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


    #[test]
    fn Str__delta__same_values() -> DeltaResult<()> {
        let s0: Str<'static> = Str::from("foo");
        let s1: Str<'static> = Str::from("foo");
        let delta: <Str as Core>::Delta = s0.delta(&s1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: {}", json_string);
        assert_eq!(json_string, "\"foo\"");
        let delta1: <Str as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        assert_eq!(delta, Str::from("foo").into_delta()?);
        Ok(())
    }

    #[test]
    fn Str__delta__different_values() -> DeltaResult<()> {
        let s0: Str<'static> = Str::from("foo");
        let s1: Str<'static> = Str::from("bar");
        let delta: <Str as Core>::Delta = s0.delta(&s1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: {}", json_string);
        assert_eq!(json_string, "\"bar\"");
        let delta1: <Str as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        assert_eq!(delta, Str::from("bar").into_delta()?);
        Ok(())
    }

    #[test]
    fn Str__apply__same_values() -> DeltaResult<()> {
        let s0: Str<'static> = Str::from("foo");
        let s1: Str<'static> = Str::from("foo");
        let delta: <Str as Core>::Delta = s0.delta(&s1)?;
        let s2 = s0.apply(delta)?;
        assert_eq!(s1, s2);
        Ok(())
    }

    #[test]
    fn Str__apply__different_values() -> DeltaResult<()> {
        let s0: Str<'static> = Str::from("foo");
        let s1: Str<'static> = Str::from("bar");
        let delta: <Str as Core>::Delta = s0.delta(&s1)?;
        let s2 = s0.apply(delta)?;
        assert_eq!(s1, s2);
        Ok(())
    }
}
