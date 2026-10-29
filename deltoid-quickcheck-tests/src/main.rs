use deltoid::{Apply, Delta};
use deltoid_derive::Delta;
use quickcheck::{quickcheck, Arbitrary, Gen};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Delta)]
#[derive(Serialize, Deserialize)]
pub struct Basic {
    pub field: Option<u32>,
}

impl Arbitrary for Basic {
    fn arbitrary(g: &mut Gen) -> Self {
        Basic { field: Arbitrary::arbitrary(g) }
    }
}

fn tester(a: Basic, b: Basic) -> bool {
    // Roundtrip property: a.apply(a.delta(&b)) == b

    println!("a: {:?}", a);
    println!("b: {:?}", b);
    let delta = a.delta(&b).unwrap();
    println!("delta: {:?}", delta);
    let c = a.apply(delta).unwrap();
    println!("c: {:?}", c);
    println!("b == c: {:?}", b == c);
    println!();
    return b == c;
}

fn main() {
    quickcheck(tester as fn(Basic, Basic) -> bool);
}
