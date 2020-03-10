# Deltoid

> The little things make all the difference

A rust library and derive macro that can be used to calculate a delta `Δ`
between 2 values of the same type, `a` and `b`.  Once calculated, `Δ` can
then be applied to the first value `a` to obtain a new value `c` that is
equivalent to the second value `b`.

A main use case for calculating delta's is to keep track of a history of
composed values (i.e. defined with `struct` or `enum`) while making sure
to keep consumption of resources (e.g. RAM, network bandwidth) reasonable.
Since a history may be exported for further processing, delta's are by
definition de/serializable. This allows gathering the data in once place
as a sequence of delta's, export it (perhaps over a network connection),
and then reconstruct the history on the receiving side by successively
applying the delta's in the sequence.


### Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
deltoid = "0.2"
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

    let point2 = point0.apply_delta(&delta).unwrap();
    assert_eq!(point1, point2);
}
```



### Limitations

There are some limitations to this library:

1. Unions are not supported. Only `struct`s and `enum`s are supported.

2. The derive macro can be used on simple generic types, but
      types making use of advanced generics are not supported.
      In such cases, it is better to manually implement the
      `Deltoid` trait for your type.
