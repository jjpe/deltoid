# Deltoid

![Rust](https://github.com/jjpe/deltoid/workflows/Rust/badge.svg)
[![](https://img.shields.io/crates/v/deltoid?label=deltoid)](https://crates.io/crates/deltoid)
[![](https://img.shields.io/crates/v/deltoid-derive?label=deltoid-derive)](https://crates.io/crates/deltoid-derive)
![](https://img.shields.io/badge/rustc-1.51+-darkcyan.svg)
![](https://img.shields.io/crates/l/deltoid)

## Synopsis

**Deltoid** is a type-driven rust library that can be used to calculate [deltas].
A delta `Δ` can be calculated between 2 values `a` and `b` of the same type.
Once calculated, `Δ` can then be applied to the first value `a` to obtain a new
value `c` that is equivalent to the second value `b`.

A primary use case for calculating delta's is to keep track of a sequence of
related and potentially deeply-nested data trees while making sure to keep
resource consumption (e.g. RAM, network bandwidth) reasonable.  Since such a
sequence may be exported for further processing, delta's are by definition
de/serializable.  This allows you to collect the data in once place as a
sequence of delta's, export it (perhaps over a network connection), and then
reconstruct the original sequence on the receiving side by successively
applying the delta's in the sequence.

[deltas]: https://en.wikipedia.org/wiki/Delta_encoding

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deltoid = "0.10"
deltoid-derive = "0.10"
```

Computing a delta, then applying it:

``` rust
use deltoid::Deltoid;
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Deltoid)]
struct Point {
    x: usize,
    y: usize,
}

fn main() {
    // Define 2 instances of the same type
    let point0 = Point { x:  0, y: 0 };
    let point1 = Point { x: 42, y: 8 };

    // Calculate the delta between them
    let delta = point0.delta(&point1).unwrap();

    // Apply  the delta to `point0`
    let point2 = point0.apply(delta).unwrap();
    assert_eq!(point1, point2);
}
```

## Limitations

There are some limitations to this library:

1. Unions are not supported. Only `struct`s and `enum`s are currently supported.

2. The derive macro tries to accommodate generic types, but for types making
   use of advanced generics a manual implementation is generally recommended
   over using `deltoid-derive` because it allows for finer control.

3. Types that have fields that have a borrow type (i.e. `&T` and `&mut T`
   for some type `T`) are not currently supported.  This limitation *may*
   be lifted in the future for mutable borrows, but is pretty fundamental
   for immutable borrows.

4. It's possible that while developing you notice that a set of impls is missing
   for a type in Rust's `stdlib`.  If so, this is because support for types that
   are a part of `stdlib` must be added manually and simply hasn't been done yet.
   You can file an issue for that, or even better, send a PR :)


## Special Thanks

A special thanks to [Accept B.V.](https://www.acc.nl/) for sponsoring this project.
