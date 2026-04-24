use crate::{CompressOptions, default_options, test, test_options, test_same_options};

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
    // Uncapped baseline: three consecutive if-return statements plus a
    // trailing return collapse into one nested ternary 3 levels deep.
    test(
        "function _() { if (a) return 1; if (b) return 2; if (c) return 3; return 4; }",
        "function _() { return a ? 1 : b ? 2 : c ? 3 : 4; }",
    );

    let opts = |n: u32| CompressOptions { max_conditional_depth: Some(n), ..default_options() };

    // Cap = 2: only the last two merges happen, producing a 2-level
    // ternary; the outermost `if (a) return 1` stays as a separate
    // statement.
    test_options(
        "function _() { if (a) return 1; if (b) return 2; if (c) return 3; return 4; }",
        "function _() { if (a) return 1; return b ? 2 : c ? 3 : 4; }",
        &opts(2),
    );

    // Cap = 0 disables if-return ternary collapse entirely; the chain
    // stays as written (separate if-returns plus trailing return).
    test_same_options("function _() { if (a) return 1; if (b) return 2; return 3; }", &opts(0));

    // Same cap applies to throw-statement chains.
    test_options(
        "function _() { if (a) throw 1; if (b) throw 2; if (c) throw 3; throw 4; }",
        "function _() { if (a) throw 1; throw b ? 2 : c ? 3 : 4; }",
        &opts(2),
    );
}
