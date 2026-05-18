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
