use std::fmt::Write;

use crate::{test, test_same};

#[test]
fn test_for_variable_declaration() {
    test(
        "function _() { var x; for (var i = 0; i < 10; i++) console.log(i) }",
        "function _() { for (var x, i = 0; i < 10; i++) console.log(i) }",
    );
    test(
        "function _() { var x = 1; for (var i = 0; i < 10; i++) console.log(i) }",
        "function _() { for (var x = 1, i = 0; i < 10; i++) console.log(i) }",
    );
    // this is fine because `let j` inside the block cannot be referenced from `var i = j`
    test(
        "function _() { var x = function () { return console.log(j), 1 }; for (var i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
        "function _() { for (var x = function () { return console.log(j), 1 }, i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
    );
    // this is fine because `let j` inside the block cannot be referenced from `var i = j`
    test(
        "function _() { var x = j; for (var i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
        "function _() { for (var x = j, i = 0; i < 10; i++) { let j = k; console.log(i, j, j) } }",
    );
}

#[test]
fn test_for_continue_in_for() {
    test("for( a of b ){ if(c) { continue; } d() }", "for ( a of b ) c || d();");
    test("for( a in b ){ if(c) { continue; } d() }", "for ( a in b ) c || d();");
    test("for( ; ; ){ if(c) { continue; } d() }", "for ( ; ; ) c || d();");

    test("for( a of b ){ c(); continue; }", "for ( a of b ) c();");
    test("for( a in b ){ c(); continue; }", "for ( a in b ) c();");
    test("for( ; ; ){ c(); continue; }", "for ( ; ; ) c();");
}

#[test]
fn test_for_in_block_scoped_no_inline() {
    // Should NOT inline when for-in uses `let` or `const` because it can cause variable shadowing
    // https://github.com/oxc-project/oxc/issues/18650
    // The inlined expression might reference a variable with the same name as the for-in variable,
    // causing it to incorrectly reference the shadowed for-in variable instead of the outer variable.
    test(
        "{ var name = 'name1'; const foo = { foo: 1 }; name = 'name2'; for (let name in foo) { console.log(name); } console.log(name); }",
        "{ var name = 'name1'; let foo = { foo: 1 }; name = 'name2'; for (let name in foo) console.log(name); console.log(name); }",
    );
    test(
        "{ var name = 'name1'; const foo = { foo: 1 }; name = 'name2'; for (const name in foo) { console.log(name); } console.log(name); }",
        "{ var name = 'name1'; let foo = { foo: 1 }; name = 'name2'; for (let name in foo) console.log(name); console.log(name); }",
    );
    test(
        "{ var name = 'name1'; const foo = { foo: 1 }; name = 'name2'; for (var name in foo) { console.log(name); } console.log(name); }",
        "var name = 'name1'; for (var name in name = 'name2', { foo: 1 }) console.log(name); console.log(name);",
    );
    test(
        "{ var name = 'name1'; const foo = { foo: 1 }; name = 'name2'; for (name in foo) { console.log(name); } console.log(name); }",
        "var name = 'name1'; for (name in name = 'name2', { foo: 1 }) console.log(name); console.log(name);",
    );
}

#[test]
fn test_max_conditional_depth_caps_return_ternary_chain() {
    let n = 600;
    let mut input = "function _() {".to_string();
    for i in 0..n {
        write!(input, "if (a{i}) return {i} + 1;").unwrap();
    }
    input.push_str("return 600; }");

    let mut output = "function _() {".to_string();
    for i in 0..99 {
        write!(output, "if (a{i}) return {};", i + 1).unwrap();
    }
    output.push_str("return a99");
    for i in 100..599 {
        write!(output, " ? {i} : a{i}").unwrap();
    }
    output.push_str(" ? 599 : (a599, 600); }");

    test(&input, &output);
}

// https://github.com/oxc-project/monitor-oxc/actions/runs/25841541741/job/75927903765
// `{ body }` and `{ body: body }` are observationally equivalent, so adjacent
// jump statements returning either form should collapse into a single `if`.
// The bug was that ObjectProperty's structural equality compares the `shorthand`
// flag, leaving the trailing `{ body: body }` outside the merged chain on the
// first pass.
#[test]
fn test_merge_adjacent_ifs_with_shorthand_object_property() {
    test(
        "function _(body) {
            if (a) return { body };
            if (b) return { body };
            if (c) return { body };
            if (d) return { body: body };
        }",
        "function _(body) { if (a || b || c || d) return { body }; }",
    );
    // String-literal key normalises to identifier first, then to shorthand —
    // both transforms must land in a single Compressor::build call.
    test(
        "function _(body) {
            if (a) return { body };
            if (b) return { 'body': body };
        }",
        "function _(body) { if (a || b) return { body }; }",
    );
    test_same(
        "function _(body, other) {
            if (a) return { body };
            if (b) return { other };
        }",
    );
}

// `{ __proto__: __proto__ }` sets `[[Prototype]]` via the Annex B.3.1 proto
// setter, while `{ __proto__ }` is a plain shorthand that creates a regular
// own data property. Normalising the former into the latter would change
// observable behaviour, so it must be left alone.
#[test]
fn test_object_property_shorthand_normalisation_skips_proto_setter() {
    test_same("function _(__proto__) { return { __proto__: __proto__ }; }");
}

#[test]
fn test_handle_switch_statement() {
    test("switch (a()) {}", "a()");
    test("switch (a) { default: }", "a;");
    test("switch (a) { default: break;}", " a;");
    test("switch (a) { default: var b; break;}", "switch (a) { default: var b; }"); // a; var b;
    test_same("switch (a) { default: b()}"); // a; b();
    test_same("switch (a) { default: b(); return;}"); // a, b(); return;

    test("switch (a) { case 1: break;}", "a;");
    test_same("switch (a) { case 1: b();}"); // a === 1 && b();
    test("switch (a) { case 1: b();break; }", "switch (a) { case 1: b(); }"); // a === 1 && b();
    test_same("switch (a) { case 1: b();return; }"); // if (a === 1) { b(); return; }

    test("switch (a) { default: case 1: }", "a;");
    test("switch (a) { case 1: default: }", "a;");
    test_same("switch (a) { case 1: default: break; case 2: b()}");
    test_same("switch (a) { case 1: b(); default: c()}"); // a === 1 && b(); c();
    test_same("switch (a) { case 1: default: b(); case 2: c();}");
    test_same("switch (a) { case 1: b(); default: break; case 2: c()}");
    test_same("switch (a) { case 1: b(); case 2: break; case 3: c()}");
    test(
        "switch (a) { case 1: b(); break; case 2: c();break;}",
        "switch (a) { case 1: b(); break; case 2: c();}",
    );
    test_same("switch (x) { default: foo(); case 1: }");
    test_same("switch (a) { case 1: b(); case 2: b();}");
    test_same("switch (a) { case 1: case 2: b(); }");
    test("switch (a) { case 1: var c=2; break;}", "switch (a) { case 1: var c=2; }"); // if (a === 1) { var c=2; }
    test("switch (a) { case 1: case 2: default: b(); break;}", "switch (a) { default: b(); }"); // a, b();

    test("switch (a) { default: break; case 1: break;}", "a;");
    test(
        "switch (a) { default: b();break;case 1: c();break;}",
        "switch (a) { default: b();break;case 1: c();}",
    ); // a === 1 ? c() : b();
    test(
        "switch (a) { default: {b();break;} case 1: {c();break;}}",
        "switch (a) { default: b();break;case 1: c(); }",
    ); // a === 1 ? c() : b();

    test("switch (a) { case b(): default:}", "switch (a) { case b(): }"); // a, b();
    test("switch (a) { case 2: case 1: break; default: break;}", "a;");
    test("switch (a) { case 3: b(); break; case 2: break;}", "switch (a) { case 3: b(); }"); // a === 3 && b();
    test("switch (a) { case 3: b(); case 2: break;}", "switch (a) { case 3: b(); }"); // a === 3 && b();
    test(
        "switch (a) { case 3: b(); case 2: c(); break;}",
        "switch (a) { case 3: b(); case 2: c(); }",
    );
    test("switch (a) { case 3: b(); case 2: case 1: break;}", "switch (a) { case 3: b(); }"); // a === 3 && b();
    test("switch (a) { case 3: b(); case 2: case 1: }", "switch (a) { case 3: b(); }"); // a === 3 && b();
    test_same("switch (x) { default: case 1: foo(); case 2: }"); // x !== 2 && foo();
    test("switch (a) { case 3: if (b) break }", "switch (a) { case 3: b; }"); // a === 3 && b;
    test("switch (a) { case 3: { if(b) {c()} else {break;} }}", "switch (a) { case 3: b && c(); }"); // a === 3 && b && c();
    test(
        "switch (a) { case 3: { if(b) {c(); break;} else { d(); break;} }}",
        "switch (a) { case 3: b ? c() : d(); }",
    ); // if (a === 3) b ? c() : d();
    test("switch (a) { case 3: { for (;;) break } }", "switch (a) { case 3: for (;;) break; }"); // if (a === 3) for (;;) break;
    test(
        "switch (a) { case 3: { for (b of c) break; } }",
        "switch (a) { case 3: for (b of c) break; }",
    ); // if (a === 3) for (b of c) break;
    test_same("switch (a) { case 3: with(b) break}");
    test("switch (a) { case 3: while(!0) break}", "switch (a) { case 3: for (;;) break; }"); // if (a === 3) while(!0) break;

    test(
        "switch (a) { case 1: c(); case 2: default: b();break;}",
        "switch (a) { case 1: c(); default: b(); }",
    ); // a === 1 && c(); b();
    test("function f() { switch (a) { case 1: return;} }", "function f() { a; }");
    test("switch (a()) { default: {let y;} }", "switch (a()) { default: { let y; } }"); // a(); { let y; }
    test(
        "function f(){switch ('x') { case 'x': var x = 1;break; case 'y': break; }}",
        "function f(){switch ('x') { case 'x': var x = 1; }}",
    );
    test("switch (a) { default: if(a) {break;}c();}", "switch (a) { default: if(a) break;c();}"); // a, !a && c();
    test("switch (a) { case 1: if(a) {b();}c();}", "switch (a) { case 1: a && b(), c(); }"); // if (a === 1) { a && b(), c(); }
    test("switch ('\\v') { case '\\u000B': foo();}", "switch ('\\v') { case '\\v': foo(); }"); // foo();

    test_same("x: switch (a) { case 1: break x;}"); // x: { a; break x; }
    test_same("x: switch (a) { case 2: break x; case 1: break x;}"); // x: { a; break x; }
    test_same("x: switch (2) { case 2: f(); break outer; }"); // x: { f(); break outer; }
    test(
        "x: switch (x) { case 2: f(); for (;;){break outer;}}",
        "x: switch (x) { case 2: for (f();;) break outer; }",
    ); // x: if (x === 2) for (f();;) break outer;
    test(
        "x: switch (a) { case 2: if(b) { break outer; } }",
        "x: switch (a) { case 2: if (b) break outer; }",
    ); // x: if (a === 2 && b) break outer;

    test(
        "switch ('r') { case 'r': a();break; case 'r': var x=0;break;}",
        "switch ('r') { case 'r': a();break; case 'r': var x=0;}",
    ); // a();
    test_same("switch (2) { default: a; case 1: b()}"); // a, b();
    test_same("switch (1) { case 1: a();break; default: b();}"); // a();
    test_same("switch ('e') { case 'e': case 'f': a();}"); // a();
    test(
        "switch ('a') { case 'a': a();break; case 'b': b();break;}",
        "switch ('a') { case 'a': a();break; case 'b': b();}",
    ); // a();
    test(
        "switch ('c') { case 'a': a();break; case 'b': b();break;}",
        "switch ('c') { case 'a': a();break; case 'b': b();}",
    ); // ;
    test(
        "switch (1) { case 1: a();break; case 2: bar();break;}",
        "switch (1) { case 1: a();break; case 2: bar();}",
    ); // a();
    test_same("switch ('f') { case 'f': a(); case 'b': b();}");
    test_same("switch ('f') { case 'f': if (a() > 0) {b();break;} c(); case 'd': f();}");
    test(
        "switch ('f') { case 'b': bar();break; case x: x();break; case 'f': f();break;}",
        "switch ('f') { case 'b': bar();break; case x: x();break; case 'f': f(); }",
    );
    test(
        "switch (1) { case 1: case 2: {break;} case 3: case 4: default: b(); break;}",
        "switch (1) { case 1: case 2: break; default: b(); }",
    );
    test(
        "switch ('d') { case 'foo': foo();break; default: bar();break;}",
        "switch ('d') { case 'foo': foo();break; default: bar();}",
    ); // bar()
    test(
        "switch (0) { case NaN: foobar();break;case -0: foo();break; case 2: bar();break;}",
        "switch (0) { case NaN: foobar();break;case -0: foo();break; case 2: bar();}",
    ); // foo()
    test(
        "let x = 1; switch ('x') { case 'x': let x = 2; break;}",
        "let x = 1; switch ('x') { case 'x': let x = 2; }",
    ); // let x = 1; { let x = 2; }
    test_same("switch (1) { case 2: var x=0;}"); // if (0) var x;
    test(
        "switch (b) { case 2: switch (a) { case 2: a();break;case 3: foo();break;}}",
        "switch (b) { case 2: switch (a) { case 2: a();break;case 3: foo();}}",
    ); // ;
    test_same("switch (b) { case 2: switch (a) { case 2: foo()}}"); // if (b === 2 && a === 2) foo()

    test_same("function f(){ switch (0) { case x: break; } let x = 1; }"); // TDZ
}
