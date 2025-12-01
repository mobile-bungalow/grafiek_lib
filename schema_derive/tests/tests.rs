#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/fixtures/test_1.rs");
    t.compile_fail("tests/fixtures/test_2.rs");
}
