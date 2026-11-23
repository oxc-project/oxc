use crate::{
    CompressOptions, test_options, test_options_with_iterations, test_smallest,
    test_unspanned_bindings,
};

#[test]
fn unreachable_unused_redeclarations() {
    for declarations in ["var x; var x", "var x, x", "{ var x } var x"] {
        test_smallest(
            &format!("export function f() {{ return; {declarations} }}"),
            "export function f() {}",
        );
    }
    test_smallest(
        "export function f() { throw 1; var x; var x }",
        "export function f() { throw 1 }",
    );
}

#[test]
fn redeclarations_with_synthetic_spans() {
    test_unspanned_bindings(
        "export function f(e) { try { x() } catch (e) { var e } return e }",
        "export function f(e) { try { x() } catch (e) { var e } return e }",
    );
    // The catch parameter and the `var` share a span, so the dropped `var` is not
    // pruned from the catch parameter's redeclarations; a second run removes it.
    test_unspanned_bindings(
        "export function f() { try { x() } catch (e) { var e } }",
        "export function f() { try { x() } catch (e) {} }",
    );
    test_unspanned_bindings(
        "export function f(e) { var e; return e }",
        "export function f(e) { var e; return e }",
    );
    test_unspanned_bindings("export function f(e) { var e }", "export function f(e) {}");
    test_unspanned_bindings("export function f(...e) { var e }", "export function f(...e) {}");
    test_unspanned_bindings(
        "export function f() { return; var x; var x }",
        "export function f() {}",
    );
    test_unspanned_bindings(
        "export function f() { try { x() } catch (e) { return; var e } return e }",
        "export function f() { try { x() } catch (e) { return; var e } return e }",
    );
    test_unspanned_bindings(
        "export function f() { try { x() } catch (e) { var e = 1; return e } }",
        "export function f() { try { x() } catch (e) { var e = 1; return e } }",
    );
    test_unspanned_bindings(
        "export function f() { try { x() } catch (e) { if (0) { try { y() } catch (e) { var e } } return e } }",
        "export function f() { try { x() } catch (e) { return e } }",
    );
    test_unspanned_bindings(
        "export function f() { try { x() } catch (e) { if (0) var e } return e }",
        "export function f() { try { x() } catch (e) { if (0) var e } return e }",
    );
    test_unspanned_bindings(
        "export function f() { var x; function x() {} }",
        "export function f() {}",
    );
}

#[test]
fn iteration_counts() {
    let options = CompressOptions::smallest();
    test_options_with_iterations("foo();", "foo();", 0, &options);
    test_options_with_iterations("let x = 1;", "", 1, &options);

    let options = CompressOptions { drop_debugger: true, ..CompressOptions::smallest() };
    test_options_with_iterations("debugger", "", 0, &options);
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
