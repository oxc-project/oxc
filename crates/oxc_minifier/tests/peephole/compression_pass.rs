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
fn reparsed_codegen_forms_do_not_request_another_pass() {
    let options = CompressOptions::smallest();
    test_options_with_iterations("foo(`plain`);", "foo('plain');", 0, &options);
    test_options_with_iterations(
        "if(a){for(;b;)if(c)throw d}else e();",
        "if(a){for(;b;)if(c)throw d}else e();",
        0,
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
