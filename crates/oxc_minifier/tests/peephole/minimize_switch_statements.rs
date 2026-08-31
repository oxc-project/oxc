use crate::test;

#[test]
fn test_handle_switch_statement_with_sequence_expressions() {
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
fn test_handle_switch_statement_with_hoisted_vars() {
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
