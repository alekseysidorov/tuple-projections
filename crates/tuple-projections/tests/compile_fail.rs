//! Compile-fail checks for invalid projections and product derives.

#[test]
fn invalid_projections_and_derives_fail_to_compile() {
    trybuild::TestCases::new().compile_fail("tests/ui/*.rs");
}
