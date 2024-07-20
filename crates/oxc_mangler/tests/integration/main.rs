mod tester;

use std::fmt::Write;

use tester::mangle;

#[test]
fn mangler() {
    let cases = [
        "function foo(a) {a}",
        "function foo(a) { let _ = { x } }",
        "function foo(a) { let { x } = y }",
        "var x; function foo(a) { ({ x } = y) }",
    ];

    let snapshot = cases.into_iter().fold(String::new(), |mut w, case| {
        write!(w, "{case}\n{}\n", mangle(case)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("mangler", snapshot);
    });
}
