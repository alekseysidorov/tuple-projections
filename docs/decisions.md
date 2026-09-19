# Design decisions

## Status

Accepted

## Date

2026-09-18

## Context

The crate needs a small, stable-Rust abstraction for expressing that one
ordinary tuple is a left prefix of another tuple:

```text
(A, B, C) = (A,) ++ (B, C)
```

The valid prefixes of `(A, B, C)` are therefore `()`, `(A,)`, `(A, B)`, and
`(A, B, C)`. `(B,)`, `(A, C)`, and `(B, C)` must not satisfy the relation.

The motivating use case is an ordered-key API such as a FoundationDB range
query, where a prefix identifies a contiguous key range. The core crate must
remain independent of FoundationDB and other database implementations.

The relation is useful as a type-level constraint:

```rust
fn find<K, P>(prefix: P)
where
    P: tuple_projections::LeftProjectionOf<K>,
{
    // The type system proves that P is a left prefix of K.
}
```

## Decision

Implement a dedicated crate with a narrow public relation:

```rust
pub trait LeftProjectionOf<T> {
    type Remainder;
}
```

`P: LeftProjectionOf<T>` means that `P` is a valid left prefix of `T`. The
associated `Remainder` records the unique suffix in `T = P ++ Remainder`.

Implementations for ordinary tuples are generated as finite, direct impls up
to arity 32. This keeps the compiler-facing output simple and avoids recursive
trait algebra or an HList representation.

The crate also provides:

- `TupleRepr` for infallible conversion between a product struct and its
  canonical tuple representation; ordinary tuples implement it as an identity
  representation;
- `#[derive(TupleProjection)]` for named, tuple, and unit structs;
- preservation of normal Rust generics, lifetimes, const generics, and
  `where` clauses.

The derive implementation is kept in the separate
`tuple-projections-derive` proc-macro crate and re-exported by the main crate.

## Alternatives considered

### `tuple_split`

`tuple_split` is the closest existing crate. Its `TupleSplitIntoLeft<L>` trait
already guarantees that `L` is the leftmost segment of a tuple, and its
type-based split works on stable Rust. It also provides runtime functions that
return the left and right values. See the
[`tuple_split` documentation](https://docs.rs/tuple_split/0.2.4/tuple_split/).

It could express the constraint as:

```rust
fn find<K, P>(prefix: P)
where
    K: tuple_split::TupleSplitIntoLeft<P>,
{
}
```

It was not adopted because the intended API needs the proposition itself, not
the operation of splitting a value. `TupleSplitIntoLeft` communicates “this
tuple can be split into this left value and a remainder”; `LeftProjectionOf`
communicates “this type is a legal left projection of that product type”. The
existing crate can encode the invariant, but it does not provide the exact
narrow domain abstraction desired here and brings additional tuple operations.

### `tupleops`

[`tupleops`](https://docs.rs/tupleops/0.1.1/tupleops/) is a broad utility
library for tuple concatenation, prepend/append, references, mapping, length,
and element-wise operations. It is useful infrastructure, but it does not
make left-prefix validity its primary relation. Depending on it would add a
larger operational surface than this crate needs.

### `tuple_list`

[`tuple_list`](https://docs.rs/tuple_list/0.1.3/tuple_list/) converts flat
tuples into recursive tuple lists such as `(A, (B, (C, ())))`, enabling
recursive variadic-style metaprogramming. That representation is useful when
implementing recursive tuple algorithms, but it introduces an intermediate
representation solely to express a relation that can be generated directly for
ordinary tuples.

### `tuplez`

[`tuplez`](https://docs.rs/tuplez/0.15.0/tuplez/) defines its own recursive
tuple type and offers indexing, split/join, subsequences, mapping, folding,
zipping, and many other operations. It solves a substantially broader problem
and requires callers to use its tuple representation rather than ordinary Rust
tuple types.

### `frunk`

[`frunk`](https://docs.rs/frunk/0.5.0/frunk/) provides HLists and generic
operations such as `Plucker` and `Sculptor`. `Sculptor` can extract a requested
shape and return a remainder, but it is concerned with type-directed HList
selection and reshaping. That is different from preserving the positional,
left-prefix relation of ordinary tuples; duplicate element types also make
type-directed selection a different problem.

### `ttools`

[`ttools`](https://docs.rs/ttools/0.1.2/ttools/) provides convenience
operations such as `head`, `last`, `take`, `drop`, and `pick`. Those operations
are adjacent to projection conceptually, but the crate is an operational tuple
toolbox rather than a small nominal relation for compile-time prefix validity.

## Scope and non-goals

The initial design intentionally excludes:

- right projections;
- arbitrary subsets or permutations;
- type-based element lookup;
- HLists or another custom tuple representation;
- recursive flattening of nested product types;
- serialization and database integration;
- runtime prefix matching;
- variadic-generic emulation beyond finite generated tuple impls.

The finite arity limit is a deliberate stable-Rust trade-off. It is centralized
in the `impl_left_projections!(32)` invocation and can be changed if a real
use case requires another limit.

## Consequences

Positive consequences:

- Generic APIs can state the prefix invariant directly.
- Invalid projections fail at compile time and have no implementation to
  accidentally select.
- The core runtime dependency surface remains empty; only the proc-macro and
  test tooling are separate workspace dependencies.
- The design remains compatible with ordinary Rust tuple types and can be used
  by a future database-specific layer without coupling this crate to a database.

Costs and limitations:

- Tuple implementations are finite and generated rather than truly variadic.
- The proc-macro crate is required for generated tuple impls and the derive
  API.
- `TupleRepr` and the derive API expand the crate beyond the smallest marker
  relation, but they provide the requested bridge for named product types
  without changing the tuple-based core model.

## Rationale

Existing libraries can encode parts of this behavior, especially `tuple_split`,
but they mostly provide operations on tuples. A dedicated
`LeftProjectionOf<T>` relation is the smallest API that names the invariant
directly and leaves splitting, range construction, and database behavior to
downstream crates.
