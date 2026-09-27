use tuple_projections::LeftProjectionOf;

fn assert_projection<P, T>()
where
    P: LeftProjectionOf<T>,
{
}

fn main() {
    assert_projection::<(u8,), (u16, u8)>();
    assert_projection::<(u8, u32), (u8, u16, u32)>();
    assert_projection::<(u16, u32), (u8, u16, u32)>();
}
