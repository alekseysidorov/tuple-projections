#![no_std]

//! Type-level left projections for tuples and product types.
//!
//! A projection is a type-level relation. If `P: LeftProjectionOf<T>`, then `P` is a
//! prefix of `T`, and `Remainder` is the suffix satisfying:
//!
//! ```text
//! T = P ++ Remainder
//! ```
//!
//! ```
//! use tuple_projections::LeftProjectionOf;
//!
//! fn assert_projection<P, T>()
//! where
//!     P: LeftProjectionOf<T>,
//! {
//! }
//!
//! assert_projection::<(u64,), (u64, String, bool)>();
//! ```
//!
//! The [`TupleProjection`] derive treats a struct as an ordered product whose tuple
//! representation follows the declaration order of its fields.

extern crate self as tuple_projections;

pub use tuple_projections_derive::{TupleProjection, impl_left_projections};

/// Relates a tuple prefix to the complete tuple it prefixes.
pub trait LeftProjectionOf<T> {
    /// The suffix remaining after removing the prefix from `T`.
    type Remainder;
}

/// Converts a product type to and from its canonical tuple representation.
pub trait TupleRepr: Sized {
    /// The ordered tuple of field types represented by `Self`.
    type Tuple;

    /// Converts this product value into its tuple representation.
    fn into_tuple(self) -> Self::Tuple;

    /// Converts a tuple representation into this product value.
    fn from_tuple(tuple: Self::Tuple) -> Self;
}

impl_left_projections!(32);
