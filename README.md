# Tuple projections

[![tests](https://github.com/alekseysidorov/tuple-projections/actions/workflows/ci.yml/badge.svg)](https://github.com/alekseysidorov/tuple-projections/actions/workflows/ci.yml)
[![MIT/Apache-2 licensed](https://img.shields.io/crates/l/tuple-projections)](./LICENSE)

`tuple-projections` provides a small type-level vocabulary for left projections
of tuples and product types. If `Whole = Prefix ++ Remainder`, then
`Prefix: LeftProjectionOf<Whole, Remainder = Remainder>`.

```rust
use tuple_projections::LeftProjectionOf;

fn assert_projection<P, T>()
where
    P: LeftProjectionOf<T>,
{
}

assert_projection::<(u64,), (u64, String, bool)>();
```

The `TupleProjection` derive gives a struct a canonical ordered tuple
representation and its corresponding left projections:

```rust
use tuple_projections::TupleProjection;

#[derive(TupleProjection)]
struct Key {
    tenant: u64,
    timestamp: i64,
    id: String,
}
```

`Key` is treated as the ordered product `(u64, i64, String)`. The crate does
not perform runtime prefix matching, serialization, or database integration.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT), at your option.
