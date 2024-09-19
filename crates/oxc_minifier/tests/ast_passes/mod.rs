mod collapse_variable_declarations;
mod dead_code_elimination;
mod fold_conditions;
mod fold_constants;
mod minimize_conditions;
mod remove_syntax;
mod reorder_constant_expression;
mod substitute_alternate_syntax;

// Oxc Integration Tests

use crate::CompressOptions;

fn test(source_text: &str, expected: &str) {
    let options = CompressOptions::default();
    crate::test(source_text, expected, options);
}

fn test_same(source_text: &str) {
    test(source_text, source_text);
}

#[test]
fn cjs() {
    // Bail `cjs-module-lexer`.
    test_same("0 && (module.exports = { version });");
}

#[test] // https://github.com/oxc-project/oxc/issues/4341
fn tagged_template() {
    test_same("(1, o.f)()");
    test_same("(1, o.f)``");
    test_same("(!0 && o.f)()");
    test_same("(!0 && o.f)``");
    test_same("(!0 ? o.f : !1)()");
    test_same("(!0 ? o.f : !1)``");

    test("foo(true && o.f)", "foo(o.f)");
    test("foo(true ? o.f : false)", "foo(o.f)");
}
