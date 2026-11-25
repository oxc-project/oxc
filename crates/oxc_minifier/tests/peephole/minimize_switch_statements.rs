use oxc_span::SourceType;

use crate::{default_options, test, test_same, test_same_options_source_type};

#[test]
fn test_drop_unreachable_switch_cases_with_sequence_expressions() {
    test("switch (c, b) { case 2: switch (a) { case 2: foo()}}", "c, b === 2 && a === 2 && foo();");
    test("switch ((a(), 1)) { case 1: b(); break; default: c(); break; }", "a(), b();");
    test("switch (1) { case 1: foo(); case (a(), 2): bar(); }", "foo(), bar();");
    test(
        "switch (2) { case (a(), 1): foo(); break; case 2: bar(); break; }",
        "switch (2) { case a(), 2: bar(); }",
    );
    test(
        "switch (3) { case (a(), 1): foo(); break; case (b(), 2): bar(); break; case 3: baz(); break; }",
        "switch (3) { case a(), b(), 3: baz(); }",
    );
    test(
        "switch ((a(), 1)) { case (b(), 1): c(); break; default: d(); break; }",
        "switch (a(), 1) { case b(), 1: c(); }",
    );
    // no case matches and there is no `default`, but the tests still run
    test(
        "switch (3) { case (a(), 1): foo(); break; case 2: bar(); break; }",
        "switch (3) { case a(), 1: }",
    );
    test(
        "switch (4) { case (a(), 1): foo(); case (b(), 2): bar(); case 3: baz(); }",
        "switch (4) { case a(), b(), 2: }",
    );
    // all tests run before `default` are run
    test("switch (3) { default: foo(); break; case (a(), 1): bar(); break; }", "a(), foo();");
    test("switch (3) { default: foo(); case (a(), 1): bar(); }", "a(), foo(), bar();");
    test(
        "switch (4) { case (a(), 1): x(); default: foo(); case (b(), 2): bar(); case (c(), 3): baz(); break; }",
        "a(), b(), c(), foo(), bar(), baz();",
    );
}

#[test]
fn test_drop_unreachable_switch_cases_with_hoisted_vars() {
    test(
        "switch (3) { case 1: var q = 1; break; case 2: w(); } return q;",
        "if (0) var q; return;",
    );
    test("switch (3) { case 1: var q = 1; break; default: w(); } return q;", "w(); var q; return;");
    test(
        "switch (3) { default: w(); break; case 1: var q = 1; } return q;",
        "switch (3) { default: w(); break; var q; } return;",
    );
    test("switch (2) { case 1: var q = 1; break; case 2: w(); } return q;", "w(); var q; return;");
    test(
        "switch (1) { case 1: foo(); break; case 2: var z = 1; } return z;",
        "switch (1) { case 1: foo(); break; var z; } return;",
    );
    test(
        "switch ('r') { case 'r': a();break; case 'r': var x=0;break;}",
        "switch ('r') { case 'r': a(); break; var x; }",
    );
}

#[test]
fn test_drop_unreachable_switch_cases_tdz_in_case() {
    test_same("switch (1) { case 1: break; case 2: let x = 1; }");
    test_same("switch (1) { case 1: break; case 2: function x() {} }");
    test_same("switch (1) { case 1: break; case 2: class C {} }");
    test(
        "switch (1) { case 2: let x = 1; case 1: console.log(x); }",
        "switch (1) { case 2: let x = 1; case 1: console.log(1); }",
    );
    test_same("switch (1) { case 2: function x() {} case 1: console.log(x); }");
}

#[test]
fn test_fold_switch_with_default_to_if() {
    test(
        "function mix(mode, a, b) { switch (mode) { case 2501: a(); break; case 2500: default: b(); } }",
        "function mix(mode, a, b) { mode === 2501 ? a() : b(); }",
    );
    test("switch (x) { case 1: a(); break; default: b(); }", "x === 1 ? a() : b();");
    test("switch (x) { default: b(); break; case 1: a(); }", "x === 1 ? a() : b();");
    test("switch (x) { case y(): a(); break; default: b(); }", "x === y() ? a() : b();");
    test("switch (x) { case 1: break; default: b(); }", "x === 1 || b();");
    test(
        "function f() { switch (x) { case 1: return a; default: b(); } }",
        "function f() { if (x === 1) return a; b(); }",
    );
    test(
        "for (;;) switch (x) { case 1: a(); continue; default: b(); }",
        "for (;;) if (x === 1) { a(); continue; } else b();",
    );
    test("x: switch (a) { case 1: b(); break x; default: c(); }", "a === 1 ? b() : c();");
    test(
        "for (;;) switch (x) { case 1: a(); break; default: b(); }",
        "for (;;) x === 1 ? a() : b();",
    );
    test(
        "switch (x) { case 1: for (;;) { if (y()) break; } a(); break; default: b(); }",
        "if (x === 1) { for (; !y();); a(); } else b();",
    );
    test(
        "function f(x, a, b) { switch (x) { case 1: a(); break; default: var y = a(); b(y); return eval('y'); } }",
        "function f(x, a, b) { if (x === 1) a(); else { var y = a(); return b(y), eval('y'); } }",
    );
    test(
        "function f(x, a, b) { switch (x) { default: var y = a(); b(y); return eval('y'); case 1: a(); } }",
        "function f(x, a, b) { if (x === 1) a(); else { var y = a(); return b(y), eval('y'); } }",
    );
    test(
        "function f(x, a, b) { switch (x) { case 1: a(); break; default: var y = a(); b(y); return () => eval('y'); } }",
        "function f(x, a, b) { if (x === 1) a(); else { var y = a(); return b(y), () => eval('y'); } }",
    );

    test_same("switch (x) { case 1: a(); default: b(); }");
    test_same("switch (x) { case 1: for (;;) break; default: b(); }");
    test_same("switch (x) { default: b(); case 1: a(); }");
    test_same("switch (x) { case 1: if (y) break; a(); break; default: b(); }");
    test_same("switch (x) { case 1: a(); break; default: if (y) break; b(); }");
    test_same(
        "switch (x) { case 1: l: { if (y) break; if (z) break l; a(); } break; default: b(); }",
    );
    test_same(
        "switch (x) { case 1: try { if (y) break; a(); } finally { c(); } break; default: b(); }",
    );
    test_same("switch (x) { case 1: let y = a(); break; default: y = b(); }");
    test_same("switch (x) { case 1: a(); break; default: function f() {} }");
    test_same("switch (x) { case 1: a(); break; case 2: b(); }");
}

#[test]
fn test_merge_identical_switch_cases() {
    test(
        "switch (x) { case 1: return a; case 2: return a; default: b(); }",
        "switch (x) { case 1: case 2: return a; default: b(); }",
    );
    test(
        "switch (x) { case 1: f(); break; case 2: f(); break; case 3: g(); }",
        "switch (x) { case 1: case 2: f(); break; case 3: g(); }",
    );
    test(
        "switch (x) { case 1: return a; case 2: return a; case 3: return a; default: b(); }",
        "switch (x) { case 1: case 2: case 3: return a; default: b(); }",
    );
    test(
        "switch (x) { case 1: f(); break; case 2: case 3: f(); break; case 4: g(); }",
        "switch (x) { case 1: case 2: case 3: f(); break; case 4: g(); }",
    );
    test(
        "switch (x) { default: return a; case 1: return a; case 2: b(); }",
        "switch (x) { default: case 1: return a; case 2: b(); }",
    );
    test(
        "switch (x) { case 1: g(); case 2: f(); break; case 3: f(); }",
        "switch (x) { case 1: g(); case 2: case 3: f(); }",
    );
    test("switch (x) { case 1: return a; default: return a; }", "return x, a;");

    test_same("switch (x) { case 1: f(); case 2: f(); }");
    test_same("switch (x) { case 1: f(); break; case 2: f(); case 3: g(); }");
    test(
        "switch (x) { case 1: f(); break; case 2: g(); break; case 3: f(); break; }",
        "switch (x) { case 1: f(); break; case 2: g(); break; case 3: f(); }",
    );
    test_same("x: { switch (a) { case 1: f(); break x; case 2: f(); } g(); }");
    test_same("for (;;) { switch (a) { case 1: f(); continue; case 2: f(); } g(); }");
    test_same("switch (x) { case 1: return tag`a`; case 2: return tag`a`; }");
    test_same("switch (x) { case 1: return () => tag`a`; case 2: return () => tag`a`; }");
    test(
        "switch (x) { case 1: var a = 0; break; case 2: var a = 0; break; } if (a) foo();",
        "switch (x) { case 1: var a = 0; break; case 2: var a = 0; } a && foo();",
    );
    test_same_options_source_type(
        "function g(x) { switch (x) { case 1: function f() {} return f; case 2: function f() {} return f; } }",
        SourceType::cjs(),
        &default_options(),
    );
}
