#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    // Note: The CkResource test is omitted because it requires full type setup with all traits
    // implemented. The macro itself is tested via the full crudkit-web workspace tests.
    t.pass("tests/cases/02-action-payload-parse.rs");
    t.pass("tests/cases/03-action-payload-use.rs");
}
