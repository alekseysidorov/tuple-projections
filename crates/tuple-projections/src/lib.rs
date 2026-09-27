#![no_std]

//! # Overview
#![doc = include_utils::include_md!("README.md:description")]
//!
//! # Tuple projections
#![doc = include_utils::include_md!("README.md:tuple_projection_example")]
//!
//! # Product types
#![doc = include_utils::include_md!("README.md:product_example")]
//!
//! # FoundationDB-style ordered keys
#![doc = include_utils::include_md!("README.md:fdb_example")]

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
