//! Generated code must not require deltoid traits in scope at the derive site.
use deltoid_derive::Delta;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Delta, Deserialize, Serialize)]
struct Hyg { a: u8, b: String }

#[derive(Clone, Debug, PartialEq, Delta, Deserialize, Serialize)]
enum HygEnum { A { x: u8 }, B(String) }

#[test]
fn derive_works_without_trait_imports() -> deltoid::DeltaResult<()> {
    let (v0, v1) = (Hyg { a: 1, b: "x".into() }, Hyg { a: 2, b: "y".into() });
    let d = deltoid::Delta::delta(&v0, &v1)?;
    assert_eq!(deltoid::Apply::apply(&v0, d)?, v1);
    let (e0, e1) = (HygEnum::A { x: 1 }, HygEnum::B("s".into()));
    let d = deltoid::Delta::delta(&e0, &e1)?;
    assert_eq!(deltoid::Apply::apply(&e0, d)?, e1);
    Ok(())
}
