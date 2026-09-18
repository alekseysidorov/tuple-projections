use tuple_projections::TupleProjection;

#[derive(TupleProjection)]
enum NotAProduct {
    One,
}

#[derive(TupleProjection)]
union AlsoNotAProduct {
    value: u8,
}

fn main() {}
