//! Runtime checks for tuple representations and type-level projections.

use tuple_projections::{LeftProjectionOf, TupleProjection, TupleRepr};

fn assert_projection<P, T, R>()
where
    P: LeftProjectionOf<T, Remainder = R>,
{
}

#[test]
fn tuple_projections_have_the_expected_remainders() {
    assert_projection::<(), (u8, u16, u32), (u8, u16, u32)>();
    assert_projection::<(u8,), (u8, u16, u32), (u16, u32)>();
    assert_projection::<(u8, u16), (u8, u16, u32), (u32,)>();
    assert_projection::<(u8, u16, u32), (u8, u16, u32), ()>();
}

#[test]
fn empty_and_single_element_tuples_are_supported() {
    assert_projection::<(), (), ()>();
    assert_projection::<(), (u8,), (u8,)>();
    assert_projection::<(u8,), (u8,), ()>();
}

#[test]
fn repeated_types_do_not_make_a_projection_ambiguous() {
    assert_projection::<(u8,), (u8, u8, u8), (u8, u8)>();
}

#[derive(Debug, PartialEq, TupleProjection)]
struct Named<A, B, C> {
    first: A,
    second: B,
    third: C,
}

#[derive(Debug, PartialEq, TupleProjection)]
struct Tuple<'a, T, const N: usize>(&'a str, T, [u8; N]);

#[derive(Debug, PartialEq, TupleProjection)]
struct Unit;

#[test]
fn named_struct_has_tuple_representation_and_projections() {
    assert_projection::<(), Named<u8, u16, u32>, (u8, u16, u32)>();
    assert_projection::<(u8,), Named<u8, u16, u32>, (u16, u32)>();
    assert_projection::<(u8, u16), Named<u8, u16, u32>, (u32,)>();
    assert_projection::<(u8, u16, u32), Named<u8, u16, u32>, ()>();

    let value = Named {
        first: 1,
        second: 2,
        third: 3,
    };
    let expected = Named {
        first: 1,
        second: 2,
        third: 3,
    };
    let tuple = value.into_tuple();
    assert_eq!(tuple, (1, 2, 3));
    assert_eq!(Named::from_tuple(tuple), expected);
}

#[test]
fn tuple_struct_preserves_lifetimes_const_generics_and_where_clauses() {
    fn convert<T, const N: usize>(value: Tuple<'_, T, N>) -> Tuple<'_, T, N>
    where
        T: Copy,
    {
        Tuple::from_tuple(value.into_tuple())
    }

    let value = Tuple::<_, 2>("prefix", 7_u16, [1, 2]);
    assert_eq!(convert(value), Tuple("prefix", 7, [1, 2]));

    assert_projection::<(&str,), Tuple<'static, u16, 2>, (u16, [u8; 2])>();
}

#[test]
fn unit_struct_has_only_the_empty_projection() {
    assert_projection::<(), Unit, ()>();
    Unit.into_tuple();
    assert_eq!(Unit::from_tuple(()), Unit);
}
