use crate::{CompressOptions, test_options, test_options_with_iterations};

#[test]
fn iteration_counts() {
    let options = CompressOptions::smallest();
    test_options_with_iterations("foo();", "foo();", 0, &options);
    test_options_with_iterations("let x = 1;", "", 1, &options);

    let options = CompressOptions { drop_debugger: true, ..CompressOptions::smallest() };
    test_options_with_iterations("debugger", "", 0, &options);
}

#[test]
fn removed_references_are_visible_within_pass() {
    let options = CompressOptions::smallest();
    test_options_with_iterations("var d = c; var c = b; var b = a; var a = 0;", "", 1, &options);
}

#[test]
fn dirty_declaration_worklist_converges_within_pass() {
    let options = CompressOptions::dce();
    test_options_with_iterations(
        "var a = 0; var b = [a, a]; var c = [b, b]; var d = [c, c];",
        "",
        1,
        &options,
    );
}

#[test]
fn dirty_declaration_worklist_handles_redeclarations() {
    let options = CompressOptions::dce();
    test_options_with_iterations(
        "var a = 0; var a = 1; var b = [a, a]; var c = [b, b];",
        "",
        1,
        &options,
    );
}

#[test]
fn normalize_flushes_before_initial_liveness() {
    let options = CompressOptions { drop_console: true, ..CompressOptions::smallest() };
    test_options("console.log(eval('x')); function f() { f() }", "", &options);

    let options = CompressOptions::smallest();
    test_options("function f() { f() } void f;", "", &options);
}

#[test]
fn dropped_direct_eval_converges_after_liveness_refresh() {
    let options = CompressOptions::smallest();
    test_options_with_iterations("if (false) eval('x'); function f() { f() }", "", 2, &options);
}
