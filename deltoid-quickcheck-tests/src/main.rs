use deltoid::{Apply, Delta};
use deltoid_derive::Delta;
use quickcheck::quickcheck;
use quickcheck_derive::Arbitrary;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Delta)]
#[derive(Serialize, Deserialize, Arbitrary)]
pub struct Basic {
    pub field: Option<u32>,
}

fn tester(a: Basic, b: Basic) -> bool {
    // NOTE: The issue seems to be that currently there is no way to
    //       tell the difference between these 2 situations after
    //       calculating `let delta = a.delta(b).unwrap()`:
    //    1. There is no `delta` because `a == b`
    //    2. There is a delta, and it happens to be `None`.
    //        This means that, when applied to a scalar value `a`,
    //        `c` will also take a value which wraps `None`.
    // Both situations are currently encoded in `delta` by having
    // it wrap `None`.

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
