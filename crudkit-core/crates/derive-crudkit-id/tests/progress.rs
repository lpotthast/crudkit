#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-parse.rs");
    t.compile_fail("tests/02-missing-id-fields.rs");
    t.pass("tests/03-uncommon-name.rs");
    t.compile_fail("tests/04-uncommon-name-no-annotation.rs");
    t.pass("tests/05-id-struct.rs");
    t.pass("tests/06-id-enum.rs");
    t.pass("tests/06b-id-enum-i64.rs");
    t.pass("tests/07-display-struct.rs");
    t.pass("tests/08-display-enum.rs");
    t.pass("tests/09-serialize-deserialize.rs");
    t.pass("tests/10-all-supported-types.rs");
    t.compile_fail("tests/11-error-optional.rs");
    t.compile_fail("tests/12-error-f32.rs");
    t.compile_fail("tests/13-error-unknown-type.rs");
    t.pass("tests/14-has-id-impl.rs");
}
