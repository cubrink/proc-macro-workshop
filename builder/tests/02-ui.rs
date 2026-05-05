//#[test]
//fn tests() {
//    let t = trybuild::TestCases::new();
//    t.pass("tests/01-parse.rs");
//    t.pass("tests/02-create-builder.rs");
//    //t.pass("tests/03-call-setters.rs");
//    //t.pass("tests/04-call-build.rs");
//    //t.pass("tests/05-method-chaining.rs");
//    //t.pass("tests/06-optional-field.rs");
//    //t.pass("tests/07-repeated-field.rs");
//    //t.compile_fail("tests/08-unrecognized-attribute.rs");
//    //t.pass("tests/09-redefined-prelude-types.rs");
//}

#[test]
fn compile_fail() {
    let _t = trybuild::TestCases::new();
    // t.compile_fail("tests/ui/fail/08-unrecognized-attribute.rs");
}

#[test]
fn compile_pass() {
    let t = trybuild::TestCases::new();
    //t.pass("tests/ui/pass/*.rs");
    t.pass("tests/ui/pass/01-parse.rs");
    t.pass("tests/ui/pass/02-create-builder.rs");
    t.pass("tests/ui/pass/03-call-setters.rs");
    //t.pass("tests/pass/04-call-build.rs");
    //t.pass("tests/pass/05-method-chaining.rs");
    //t.pass("tests/pass/06-optional-field.rs");
    //t.pass("tests/pass/07-repeated-field.rs");
    //t.pass("tests/pass/09-redefined-prelude-types.rs");
}
