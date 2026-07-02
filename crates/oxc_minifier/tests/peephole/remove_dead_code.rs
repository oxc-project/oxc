use oxc_span::SourceType;

use crate::{
    CompressOptions, CompressOptionsUnused, default_options, test, test_options, test_same,
    test_same_options, test_same_options_source_type,
};

#[track_caller]
fn test_unused(source_text: &str, expected: &str) {
    let options = CompressOptions { unused: CompressOptionsUnused::Remove, ..default_options() };
    test_options(source_text, expected, &options);
}

#[test]
fn test_fold_block() {
    test("{{foo()}}", "foo()");
    test("{foo();{}}", "foo()");
    test("{{foo()}{}}", "foo()");
    test("{{foo()}{bar()}}", "foo(), bar()");
    test("{if(false)foo(); {bar()}}", "bar()");
    test("{if(false)if(false)if(false)foo(); {bar()}}", "bar()");

    test("{'hi'}", "");
    test("{x==3}", "x");
    test("{`hello ${foo}`}", "`${foo}`");
    test("{ (function(){x++}) }", "");
    test("{ (function foo(){x++; foo()}) }", "");
    test("function f(){return;}", "function f(){}");
    test("function f(){return 3;}", "function f(){return 3}");
    test("function f(){if(x)return; x=3; return; }", "function f(){ x ||= 3; }");
    test("{x=3;;;y=2;;;}", "x=3, y=2");

    // Cases to test for empty block.
    // test("while(x()){x}", "while(x());");
    test("while(x()){x()}", "for(;x();)x()");
    // test("for(x=0;x<100;x++){x}", "for(x=0;x<100;x++);");
    // test("for(x in y){x}", "for(x in y);");
    // test("for (x of y) {x}", "for(x of y);");
    test("for (let x = 1; x <10; x++ ) {}", "for (let x = 1; x <10; x++ );");
    test("for (var x = 1; x <10; x++ ) {}", "for (var x = 1; x <10; x++ );");
    test("do { } while (true)", "do;while(!0)");
    test(
        "function z(a) {
          {
            for (var i = 0; i < a; i++) {}
            foo()
          }
          bar()
        }",
        "function z(a) {
          for (var i = 0; i < a; i++);
          foo(), bar()
        }",
    );
}

#[test]
fn test_remove_no_op_labelled_statement() {
    test("a: break a;", "");
    test("a: { break a; }", "");

    test("a: { break a; console.log('unreachable'); }", "");
    test("a: { break a; var x = 1; } x = 2;", "var x = 2;");

    test("b: { var x = 1; } x = 2;", "b: var x = 1; x = 2;");
    test("a: b: { var x = 1; } x = 2;", "a: b: var x = 1; x = 2;");
    test("foo:;", "");
}

#[test]
fn test_fold_useless_for() {
    test("for(;false;) { foo() }", "");
    test("for(;void 0;) { foo() }", "");
    test("for(;undefined;) { foo() }", "");
    test("for(;true;) foo() ", "for(;;) foo() ");
    test_same("for(;;) foo()");
    test("for(;false;) { var a = 0; }", "var a");
    test("for(;false;) { const a = 0; }", "");
    test("for(;false;) { let a = 0; }", "");

    // Make sure it plays nice with minimizing
    test("for(;false;) { foo(); continue }", "");

    test("for (var { c, x: [d] } = {}; 0;);", "var { c, x: [d] } = {};");
    test("for (var se = [1, 2]; false;);", "var se = [1, 2];");
    test("for (var se = [1, 2]; false;) { var a = 0; }", "var se = [1, 2], a;");

    test("for (foo = bar; false;) {}", "for (foo = bar; !1;);");
    // test("l1:for(;false;) {  }", "");
}

#[test]
fn test_minimize_loop_with_constant_condition_vanilla_for() {
    test("for(;true;) foo()", "for(;;) foo()");
    test("for(;0;) foo()", "");
    test("for(;0.0;) foo()", "");
    test("for(;NaN;) foo()", "");
    test("for(;null;) foo()", "");
    test("for(;undefined;) foo()", "");
    test("for(;'';) foo()", "");
}

#[test]
fn test_fold_try_statement() {
    test("try { throw 0 } catch (e) { foo() }", "try { throw 0 } catch { foo() }");
    test("try {} catch (e) { var foo }", "try {} catch { var foo }");
    test("try {} catch (e) { var foo; bar() } finally {}", "try {} catch { var foo }");
    test(
        "try {} catch (e) { var foo; bar() } finally { baz() }",
        "try {} catch { var foo } finally { baz() }",
    );
    test("try {} catch (e) { foo() }", "");
    test("try {} catch (e) { foo() } finally {}", "");
    test("try {} finally { foo() }", "foo()");
    test("try {} catch (e) { foo() } finally { bar() }", "bar()");
    test("try {} finally { var x = foo() }", "var x = foo()");
    test("try {} catch (e) { foo() } finally { var x = bar() }", "var x = bar()");
    test("try {} finally { let x = foo() }", "{ let x = foo() }");
    test("try {} catch (e) { foo() } finally { let x = bar() }", "{ let x = bar();}");
    test("try {} catch (e) { } finally {}", "");
    test("try { foo() } catch (e) { bar() } finally {}", "try { foo() } catch { bar() }");
    test_same("try { foo() } catch { bar() } finally { baz() }");

    // Leak regression: when the empty `try` drops, the catch arm's write-ref
    // to `x` must be walked into `PassDirty`, else the stale write blocks
    // constant inlining of `x`.
    let options = CompressOptions::smallest();
    test_options(
        "let x = 'initial'; try {} catch (e) { x = 'unexpected'; } console.log(x);",
        "console.log('initial');",
        &options,
    );
}

#[test]
fn test_fold_if_statement() {
    test("if (foo) {}", "foo");
    test("if (foo) {} else {}", "foo");
    test("if (false) {}", "");
    test("if (true) {}", "");
    test("if (false) { var a; console.log(a) }", "if (0) var a");
    test_unused("if (false) { var a; console.log(a) }", "");
}

#[test]
fn test_fold_conditional() {
    test("true ? foo() : bar()", "foo()");
    test("false ? foo() : bar()", "bar()");
    test_same("foo() ? bar() : baz()");
    test("foo && false ? foo() : bar()", "(foo, bar());");

    test("var a; (true ? a : 0)()", "var a; a()");
    test("var a; (true ? a.b : 0)()", "var a; (0, a.b)()");
    test("var a; (false ? 0 : a)()", "var a; a()");
    test("var a; (false ? 0 : a.b)()", "var a; (0, a.b)()");
}

#[test]
fn test_remove_empty_static_block() {
    test("class Foo { static {}; foo }", "class Foo { foo }");
    test_same("class Foo { static { foo() } }");
}

#[test]
fn keep_module_syntax() {
    test_same("throw foo; export let bar");
    test_same("throw foo; export default bar");
}

#[test]
fn remove_empty_spread_arguments() {
    test("foo(...[])", "foo()");
    test("new Foo(...[])", "new Foo()");
}

#[test]
fn remove_unreachable() {
    test("while(true) { break a; unreachable;}", "for(;;) break a");
    test("while(true) { continue a; unreachable;}", "for(;;) continue a");
    test("while(true) { throw a; unreachable;}", "for(;;) throw a");
    test("while(true) { return a; unreachable;}", "for(;;) return a");

    // A kept function declaration (not a dead IIFE) so the unreachable `var a`
    // after `return` is preserved under `unused: Keep`.
    test("function f() { return; var a }", "function f() { return; var a }");
    test_unused("(function () { return; var a })()", "");
}

#[test]
fn remove_unused_expressions_in_sequence() {
    test("true, foo();", "foo();");
    test("(0, foo)();", "foo();");
    test("(0, foo)``;", "foo``;");
    test("(0, foo)?.();", "foo?.();");
    test_same("(0, eval)();"); // this can be compressed to `eval?.()`
    test_same("(0, eval)``;"); // this can be compressed to `eval?.()`
    test_same("(0, eval)?.();"); // this can be compressed to `eval?.()`
    test("var eval; (0, eval)();", "var eval; eval();");
    test_same("(0, foo.bar)();");
    test_same("(0, foo.bar)``;");
    test_same("(0, foo.bar)?.();");
    test("(true, foo.bar)();", "(0, foo.bar)();");
    test("(true, true, foo.bar)();", "(0, foo.bar)();");
    test("var foo; (true, foo.bar)();", "var foo; (0, foo.bar)();");
    test("var foo; (true, true, foo.bar)();", "var foo; (0, foo.bar)();");

    // Regression: a >=3 element sequence in indirect-access position whose
    // second-to-last element is already `0` must converge. Re-wrapping the
    // already-`0` element re-records a mutation every iteration, spinning
    // the fixed-point loop into the 10-iteration debug_assert.
    test_same("(sideEffect(), 0, foo.bar)();");
    test_same("delete (sideEffect(), 0, foo.bar);");

    test("typeof (0, foo);", "foo");
    test_same("v = typeof (0, foo);");
    test("var foo; typeof (0, foo);", "var foo;");
    test("var foo; v = typeof (0, foo);", "var foo; v = typeof foo");
    test("typeof 0", "");

    test_same("delete (0, foo);");
    test_same("delete (0, foo.#bar);");
    test_same("delete (0, foo.bar);");
    test_same("delete (0, foo[bar]);");
    test_same("delete (0, foo?.bar);");
}

#[test]
fn remove_unused_expressions_in_for() {
    test("var i; for (i = 0, 0; i < 10; i++) foo(i);", "var i; for (i = 0; i < 10; i++) foo(i);");
    test(
        "var i; for (i = 0; i < 10; 0, i++, 0) foo(i);",
        "var i; for (i = 0; i < 10; i++) foo(i);",
    );
}

#[test]
fn remove_constant_value() {
    test("const foo = false; if (foo) { console.log('foo') }", "const foo = !1;");
}

#[test]
fn remove_empty_function() {
    let options = CompressOptions::smallest();
    test_options("function foo() {} foo()", "", &options);
    test_options("function foo() {} foo(); foo()", "", &options);
    test_options("var foo = () => {}; foo()", "", &options);
    test_options("var foo = () => {}; foo(a)", "a", &options);
    test_options("var foo = () => {}; foo(a, b)", "a, b", &options);
    test_options("var foo = () => {}; foo(...a, b)", "[...a], b", &options);
    test_options("var foo = () => {}; foo(...a, ...b)", "[...a], [...b]", &options);
    test_options("var foo = () => {}; x = foo()", "x = void 0", &options);
    test_options("var foo = () => {}; x = foo(a(), b())", "x = (a(), b(), void 0)", &options);
    test_options("var foo = function () {}; foo()", "", &options);

    test_same_options("function foo({}) {} foo()", &options);
    test_options("var foo = ({}) => {}; foo()", "(({}) => {})()", &options);
    test_options("var foo = function ({}) {}; foo()", "(function ({}) {})()", &options);

    test_same_options("async function foo({}) {} foo()", &options);
    test_options("var foo = async ({}) => {}; foo()", "(async ({}) => {})()", &options);
    test_options("var foo = async function ({}) {}; foo()", "(async function ({}) {})()", &options);

    test_same_options("function* foo({}) {} foo()", &options);
    test_options("var foo = function*({}) {}; foo()", "(function*({}) {})()", &options);
}

#[test]
fn redeclared_pure_function_is_not_folded_var() {
    // `var foo` redeclarations are span-only in oxc_semantic and create no references,
    // so the read-only-refs gate on the first declarator can't see the second one.
    // Whichever declaration wins at runtime (here, the `if` branch, when truthy) may
    // be impure, so a stale "pure" fact recorded for an earlier declarator must not
    // survive to fold the call.
    test_same("var foo = (u) => {}; if (g) var foo = (a) => { console.log(a); }; foo('x');");
}

#[test]
fn redeclared_pure_function_is_not_folded_function_declaration() {
    // Duplicate `function` declarations are legal in sloppy scripts (SyntaxError in
    // modules); the second declaration wins at runtime.
    test_same_options_source_type(
        "function foo(u) {} function foo(a) { console.log(a); } foo('x');",
        SourceType::cjs().with_script(true),
        &default_options(),
    );
}

#[test]
fn non_redeclared_pure_function_still_folds() {
    // Regression guard for `redeclared_pure_function_is_not_folded`: a single,
    // un-redeclared pure function must still fold as before.
    test("const foo = (u) => {}; foo(1)", "const foo = (u) => {};");
}

// ── drop dead trailing arguments to functions that ignore them (#23866) ──

#[test]
fn drop_dead_args_issue_repro() {
    // The declared argument is never read, so the call keeps `await`/the call
    // itself but drops the object argument.
    test(
        "const foo = async (assets) => ({}); export default await foo({ bar: 'baz' })",
        "const foo = async (assets) => ({}); export default await foo()",
    );
}

#[test]
fn drop_dead_args_multi_use() {
    // Side-effectful body keeps both calls; the unused param is dropped at each.
    // (The minifier then merges the two statements into a comma sequence.)
    test(
        "const foo = (a) => { bar() }; foo(1); foo(2)",
        "const foo = (a) => { bar() }; foo(), foo()",
    );
}

#[test]
fn drop_dead_args_extra_beyond_params_and_used_param_kept() {
    // `a` is used, so arg 0 stays; args beyond the single param are dropped.
    test(
        "const foo = (a) => { bar(a) }; foo(1, 2, 3); foo(4)",
        "const foo = (a) => { bar(a) }; foo(1), foo(4)",
    );
}

#[test]
fn drop_dead_args_zero_params() {
    test(
        "const foo = () => { bar() }; foo(1, 2); foo(3)",
        "const foo = () => { bar() }; foo(), foo()",
    );
}

#[test]
fn drop_dead_args_stops_at_side_effect() {
    // The side-effectful arg must still run, so the pop stops there.
    test(
        "const foo = (a, b) => { bar() }; foo(baz(), 1); foo(2)",
        "const foo = (a, b) => { bar() }; foo(baz()), foo()",
    );
}

#[test]
fn drop_dead_args_async_effectful_unused_result_not_deleted() {
    // Consumer-2 regression: the entry carries only a `dead_arg_prefix` fact, so
    // the unused-result calls must NOT be treated as pure and deleted. Two call
    // sites keep `foo` from being inlined so the calls remain observable here.
    test(
        "const foo = async (u) => { g() }; foo(1); foo(2)",
        "const foo = async (u) => { g() }; foo(), foo()",
    );
}

#[test]
fn drop_dead_args_generator() {
    // Function expressions are strict in a module, so the non-arrow gate passes.
    test(
        "const foo = function* (u) { yield g() }; foo(1).next(); foo(2).next()",
        "const foo = function* (u) { yield g() }; foo().next(), foo().next()",
    );
}

#[test]
fn drop_dead_args_arrow_with_nested_non_arrow_arguments() {
    // The inner `arguments` belongs to the nested non-arrow, not the arrow callee.
    test(
        "const foo = (u) => function() { return arguments.length }; g(foo(1)); g(foo(2))",
        "const foo = (u) => function() { return arguments.length }; g(foo()), g(foo())",
    );
}

#[test]
fn drop_dead_args_arrow_reads_enclosing_arguments() {
    // The program mentions `arguments`, but the callee is an arrow so the gate is
    // skipped: the `arguments` belongs to `outer`.
    test(
        "function outer() { const foo = (u) => arguments.length; return foo(1) + foo(2) }",
        "function outer() { let foo = (u) => arguments.length; return foo() + foo() }",
    );
}

#[test]
fn drop_dead_args_frees_dropped_ref_same_run() {
    // Dropping the `x` args must free `x` so its declaration is removed too (needs
    // `unused: Remove` to observe the declaration removal).
    test_unused(
        "const foo = (u) => { bar() }; const x = 1; foo(x); foo(x)",
        "const foo = (u) => { bar() }; foo(), foo()",
    );
}

#[test]
fn drop_dead_args_hoisted_decl_call_before_definition() {
    // The call precedes the hoisted definition, so the callee is only recorded
    // after the call is first visited; the drop lands on the next fixed-point
    // pass via map persistence. The dead `if (0)` guarantees a first-pass
    // mutation so that second pass runs.
    test("if (0) dead(); foo(1); function foo(u) { bar() }", "foo(); function foo(u) { bar() }");
}

#[test]
fn drop_dead_args_export_named_after_decl() {
    test(
        "const foo = (u) => { bar() }; export { foo }; foo(1); foo(2)",
        "const foo = (u) => { bar() }; export { foo }; foo(), foo()",
    );
}

// Negatives: the bail must hold. Each is written with `test` (not `test_same`)
// because the minifier merges the two call statements into a comma sequence;
// the point is that the ARGUMENTS are unchanged.

#[test]
fn drop_dead_args_middle_dead_arg_kept() {
    // `a` precedes the used `b`, so it cannot be dropped from the middle.
    test(
        "const foo = (a, b) => { bar(b) }; foo(1, 2); foo(3, 4)",
        "const foo = (a, b) => { bar(b) }; foo(1, 2), foo(3, 4)",
    );
}

#[test]
fn drop_dead_args_side_effectful_sole_arg_kept() {
    test(
        "const foo = (u) => { bar() }; foo(baz()); foo(2)",
        "const foo = (u) => { bar() }; foo(baz()), foo()",
    );
}

#[test]
fn drop_dead_args_default_param_not_dropped() {
    // A parameter default runs on `undefined`; dropping args here could change it.
    test(
        "const foo = (u = g()) => { bar() }; foo(1); foo(2)",
        "const foo = (u = g()) => { bar() }; foo(1), foo(2)",
    );
}

#[test]
fn drop_dead_args_destructuring_param_not_dropped() {
    test(
        "const foo = ({ u }) => { bar() }; foo(x); foo(y)",
        "const foo = ({ u }) => { bar() }; foo(x), foo(y)",
    );
}

#[test]
fn drop_dead_args_rest_param_not_dropped() {
    test(
        "const foo = (...args) => { console.log(args) }; foo(1); foo(2)",
        "const foo = (...args) => { console.log(args) }; foo(1), foo(2)",
    );
}

#[test]
fn drop_dead_args_non_arrow_own_arguments_not_dropped() {
    test(
        "const foo = function(u) { return arguments.length }; g(foo(1)); g(foo(1, 2))",
        "const foo = function(u) { return arguments.length }; g(foo(1)), g(foo(1, 2))",
    );
}

#[test]
fn drop_dead_args_spread_before_trailing_not_dropped() {
    // A spread shifts later argument positions, so trailing pop is unsound.
    test(
        "const foo = (a, b) => { console.log(a) }; foo(...xs, 1); foo(...xs, 2)",
        "const foo = (a, b) => { console.log(a) }; foo(...xs, 1), foo(...xs, 2)",
    );
}

#[test]
fn drop_dead_args_trailing_spread_not_dropped() {
    test(
        "const foo = (u) => { bar() }; foo(...xs); foo(...ys)",
        "const foo = (u) => { bar() }; foo(...xs), foo(...ys)",
    );
}

#[test]
fn drop_dead_args_reassigned_binding_not_dropped() {
    // A writable binding may hold a different function at the call site.
    test(
        "let foo = (u) => { bar() }; foo(1); foo = g",
        "let foo = (u) => { bar() }; foo(1), foo = g",
    );
}

#[test]
fn drop_dead_args_exported_decl_not_dropped() {
    // The recorder has no `export const` arm; documents current behavior.
    test_same("export const foo = (u) => { bar() }; foo(1)");
}

#[test]
fn drop_dead_args_direct_eval_not_dropped() {
    test(
        "const foo = (u) => { bar() }; foo(1); foo(2); eval('x')",
        "const foo = (u) => { bar() }; foo(1), foo(2), eval('x')",
    );
}

#[test]
fn drop_dead_args_script_root_scope_not_dropped() {
    // A root-scope binding in a script is aliased on `globalThis`.
    test_same_options_source_type(
        "function foo(u) { bar() } foo(1)",
        SourceType::cjs().with_script(true),
        &default_options(),
    );
}

#[test]
fn drop_dead_args_sloppy_var_arguments_not_dropped() {
    // A top-level script function bails on the script-root gate before the
    // `arguments` gate is even reached; still, no drop.
    test_same_options_source_type(
        "function foo(u) { var arguments; return arguments.length } g(foo(1))",
        SourceType::cjs().with_script(true),
        &default_options(),
    );
}

#[test]
fn drop_dead_args_nested_sloppy_var_arguments_not_dropped() {
    // A nested (non-root) sloppy `function foo` with `var arguments`: the
    // `var arguments` binds a real symbol, so `arguments` is NOT an unresolved
    // reference and the program-wide lookup would miss it — the strict-mode gate
    // is what closes the hole (a sloppy scope is not strict, so it bails).
    test_same_options_source_type(
        "function outer() { function foo(u) { var arguments; return arguments } return g(foo(1)) + g(foo(1, 2)) } outer()",
        SourceType::cjs().with_script(true),
        &default_options(),
    );
}
