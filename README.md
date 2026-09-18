# Tuple projections

[![tests](https://github.com/alekseysidorov/tuple-projections/actions/workflows/ci.yml/badge.svg)](https://github.com/alekseysidorov/tuple-projections/actions/workflows/ci.yml)
[![MIT/Apache-2 licensed](https://img.shields.io/crates/l/tuple-projections)](./LICENSE)

<!-- ANCHOR: description -->
`tuple-projections` provides type-safe left projections for tuples and product
types. If `Whole = Prefix ++ Remainder`, then
`Prefix: LeftProjectionOf<Whole, Remainder = Remainder>`.

The crate is useful when a type-level prefix relation should be checked by the
compiler, for example while defining ordered key prefixes for FoundationDB-
style tuple keys. It provides the relation and tuple conversions, but leaves
encoding, serialization, and database access to the surrounding application.
<!-- ANCHOR_END: description -->

## Tuple projections

<!-- ANCHOR: tuple_projection_example -->
```rust
use tuple_projections::LeftProjectionOf;

fn assert_projection<P, T, R>()
where
    P: LeftProjectionOf<T, Remainder = R>,
{
}

type Whole = (u64, String, bool);

assert_projection::<(), Whole, Whole>();
assert_projection::<(u64,), Whole, (String, bool)>();
assert_projection::<(u64, String), Whole, (bool,)>();
assert_projection::<Whole, Whole, ()>();
```
<!-- ANCHOR_END: tuple_projection_example -->

## Product types

<!-- ANCHOR: product_example -->
The `TupleProjection` derive gives a struct a canonical ordered tuple
representation and its corresponding left projections:

```rust
use tuple_projections::{LeftProjectionOf, TupleProjection, TupleRepr};

#[derive(TupleProjection)]
struct EventKey {
    tenant: u64,
    timestamp: i64,
    id: String,
}

fn assert_projection<P, T, R>()
where
    P: LeftProjectionOf<T, Remainder = R>,
{
}

assert_projection::<(u64,), EventKey, (i64, String)>();
assert_projection::<(u64, i64), EventKey, (String,)>();

let key = EventKey {
    tenant: 42,
    timestamp: 1_700_000_000,
    id: String::from("event-7"),
};
assert_eq!(
    key.into_tuple(),
    (42, 1_700_000_000, String::from("event-7")),
);
```
<!-- ANCHOR_END: product_example -->

`EventKey` is treated as the ordered product `(u64, i64, String)`, following
the declaration order of its fields.

## FoundationDB-style ordered keys

<!-- ANCHOR: fdb_example -->
FoundationDB-style keyspaces often use an ordered tuple as a key and a shorter
tuple as a key prefix. The projection trait lets an API express that a prefix
belongs to a particular full-key shape without introducing a runtime matcher:

```rust
use tuple_projections::LeftProjectionOf;

fn range_for_prefix<Key, Prefix>(_prefix: Prefix)
where
    Prefix: LeftProjectionOf<Key>,
{
    // Build the database range using the application's tuple encoder.
}

type EventKey = (u64, i64, String);
range_for_prefix::<EventKey, _>((42_u64,));
range_for_prefix::<EventKey, _>((42_u64, 1_700_000_000_i64));
```

This crate intentionally does not depend on FoundationDB or any particular
encoding format. It only describes the compile-time relationship that an
ordered-key adapter can use.
<!-- ANCHOR_END: fdb_example -->

More design context and comparisons with related crates are recorded in
[`docs/decisions.md`](docs/decisions.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.
