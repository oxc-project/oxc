use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames, Mangler};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn mangle_with_source_type(
    source_text: &str,
    options: MangleOptions,
    source_type: SourceType,
) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty(), "Parser errors: {:?}", ret.errors);
    let program = ret.program;
    let mangler_return = Mangler::new().with_options(options).build(&program);
    Codegen::new()
        .with_scoping(Some(mangler_return.scoping))
        .with_private_member_mappings(Some(mangler_return.class_private_mappings))
        .build(&program)
        .code
}

fn mangle(source_text: &str, options: MangleOptions) -> String {
    mangle_with_source_type(source_text, options, SourceType::mjs().with_unambiguous(true))
}

fn mangle_script(source_text: &str, options: MangleOptions) -> String {
    mangle_with_source_type(source_text, options, SourceType::script())
}

fn test(source_text: &str, expected: &str, options: MangleOptions) {
    let mangled = mangle(source_text, options);
    let expected = {
        let allocator = Allocator::default();
        let source_type = SourceType::mjs().with_unambiguous(true);
        let ret = Parser::new(&allocator, expected, source_type).parse();
        assert!(ret.errors.is_empty(), "Parser errors: {:?}", ret.errors);
        Codegen::new().build(&ret.program).code
    };
    assert_eq!(
        mangled, expected,
        "\nfor source\n{source_text}\nexpect\n{expected}\ngot\n{mangled}"
    );
}

#[test]
fn direct_eval() {
    let options = MangleOptions::default();

    // Symbols in scopes with direct eval should NOT be mangled
    let source_text = "function foo() { let NO_MANGLE; eval('') }";
    let mangled = mangle(source_text, options);
    assert_eq!(mangled, "function foo() {\n\tlet NO_MANGLE;\n\teval(\"\");\n}\n");

    // Nested direct eval: parent scope also should not mangle
    let source_text = "function foo() { let NO_MANGLE; function bar() { eval('') } }";
    let mangled = mangle(source_text, options);
    assert_eq!(
        mangled,
        "function foo() {\n\tlet NO_MANGLE;\n\tfunction bar() {\n\t\teval(\"\");\n\t}\n}\n"
    );

    // Sibling scope without direct eval should be mangled
    let source_text =
        "function foo() { let NO_MANGLE; eval('') } function bar() { let SHOULD_MANGLE; }";
    let mangled = mangle(source_text, options);
    // SHOULD_MANGLE gets mangled (to some short name), NO_MANGLE stays as is
    assert!(mangled.contains("NO_MANGLE"));
    assert!(!mangled.contains("SHOULD_MANGLE"));

    // Child function scope without direct eval CAN be mangled (eval in parent cannot access child function locals)
    let source_text = "function foo() { eval(''); function bar() { let CAN_MANGLE; } }";
    let mangled = mangle(source_text, options);
    assert!(!mangled.contains("CAN_MANGLE"));

    // Indirect eval should still allow mangling
    let source_text = "function foo() { let SHOULD_MANGLE; (0, eval)('') }";
    let mangled = mangle(source_text, options);
    assert!(!mangled.contains("SHOULD_MANGLE"));

    test(
        r#"var e = () => {}; var foo = (bar) => e(bar); var pt = (() => { eval("") })();"#,
        r#"var e = () => {}; var foo = (t) => e(t); var pt = (() => { eval(""); })();"#,
        MangleOptions::default(),
    );

    test(
        r#"var e = () => {}; var foo = (bar) => e(bar); var pt = (() => { eval("") })();"#,
        r#"var e = () => {}; var foo = (t) => e(t); var pt = (() => { eval(""); })();"#,
        MangleOptions { top_level: Some(true), ..MangleOptions::default() },
    );

    test(
        r"function outer() { let e = 1; eval(''); function inner() { let longNameToMangle = 2; console.log(e); } }",
        r#"function outer() { let e = 1; eval(""); function inner() { let t = 2; console.log(e); } }"#,
        MangleOptions::default(),
    );

    test(
        r"function evalScope() { let x = 1; eval(''); } function siblingScope() { let longName = 2; console.log(longName); }",
        r#"function evalScope() {let x = 1; eval(""); } function siblingScope() { let e = 2; console.log(e); }"#,
        MangleOptions::default(),
    );
}

#[test]
fn mangler() {
    let cases = [
        "function foo(a) {a}",
        "function foo(a) { let _ = { x } }",
        "function foo(a) { let { x } = y }",
        "var x; function foo(a) { ({ x } = y) }",
        "import { x } from 's'; export { x }",
        "Object.defineProperty(exports, '__esModule', { value: true })",
        "var exports = {}; Object.defineProperty(exports, '__esModule', { value: true })",
        "function _(exports) { Object.defineProperty(exports, '__esModule', { value: true }) }",
        "function _() { console.log(arguments) }",
        "function foo(foo_a, foo_b, foo_c) {}; function bar(bar_a, bar_b, bar_c) {}", // foo_a and bar_a can be reused
        "function _() { function foo() { var x; foo; } }", // x should not use the same name with foo
        "function _() { var x; function foo() { var y; function bar() { x } } }", // y should not shadow x
        "function _() { function x(a) {} }",                                      // a can shadow x
        "function _() { function x(a) { x } }", // a should not shadow x
        "function _() { var x; { var y }}",     // y should not shadow x
        "function _() { var x; { let y }}",     // y can shadow x
        "function _() { let x; { let y }}",     // y can shadow x
        "function _() { var x; { const y = 1 }}", // y can shadow x
        "function _() { let x; { const y = 1 }}", // y can shadow x
        "function _() { var x; { class Y{} }}", // Y can shadow x
        "function _() { let x; { class Y{} }}", // Y can shadow x
        "function _() { var x; try { throw 0 } catch (e) { e } }", // e can shadow x
        "function _() { var x; try { throw 0 } catch (e) { var e } }", // e can shadow x (not implemented)
        "function _() { var x; try { throw 0 } catch { var e } }",     // e should not shadow x
        "function _() { var x; var y; }", // x and y should have different names
        "function _() { var x; let y; }", // x and y should have different names
        "function _() { { var x; var y; } }", // x and y should have different names
        "function _() { { var x; let y; } }", // x and y should have different names
        "function _() { let a; { let b; { let c; { let d; var x; } } } }",
        "function _() { let a; { let b; { let c; { console.log(a); let d; var x; } } } }",
        "function _() {
          if (bar) var a = 0;
          else {
            let b = 0;
            var a = 1;
          }
        }", // a and b should have different names
    ];
    let top_level_cases = [
        "function foo(a) {a}",
        "export function foo() {}; foo()",
        "export default function foo() {}; foo()",
        "export const foo = 1; foo",
        "const foo = 1; foo; export { foo }",
    ];
    let keep_name_cases = [
        "function _() { function foo() { var x } }",
        "function _() { var foo = function() { var x } }",
        "function _() { var foo = () => { var x } }",
        "function _() { class Foo { foo() { var x } } }",
        "function _() { var Foo = class { foo() { var x } } }",
    ];

    let mut snapshot = String::new();
    cases.into_iter().fold(&mut snapshot, |w, case| {
        let options = MangleOptions::default();
        write!(w, "{case}\n{}\n", mangle(case, options)).unwrap();
        w
    });
    top_level_cases.into_iter().fold(&mut snapshot, |w, case| {
        let options = MangleOptions { top_level: Some(true), ..MangleOptions::default() };
        write!(w, "{case}\n{}\n", mangle(case, options)).unwrap();
        w
    });
    keep_name_cases.into_iter().fold(&mut snapshot, |w, case| {
        let options = MangleOptions {
            keep_names: MangleOptionsKeepNames::all_true(),
            ..MangleOptions::default()
        };
        write!(w, "{case}\n{}\n", mangle(case, options)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("mangler", snapshot);
    });
}

#[test]
fn private_member_mangling() {
    let cases = [
        "class Foo { #privateField = 1; method() { return this.#privateField; } }",
        "class Foo { #a = 1; #b = 2; method() { return this.#a + this.#b; } }",
        "class Foo { #method() { return 1; } publicMethod() { return this.#method(); } }",
        "class Foo { #field; #method() { return this.#field; } get() { return this.#method(); } }",
        "class Foo { #x; check() { return #x in this; } }",
        // Nested classes
        "class Outer { #outerField = 1; inner() { return class Inner { #innerField = 2; get() { return this.#innerField; } }; } }",
        "class Outer { #shared = 1; getInner() { let self = this; return class { method() { return self.#shared; } }; } }",
        "class Outer { #shared = 1; getInner() { return class { #shared = 2; method() { return this.#shared; } }; } }",
        // Mixed public and private
        "class Foo { publicField = 1; #privateField = 2; getSum() { return this.publicField + this.#privateField; } }",
        // Test same names across different classes should reuse mangled names
        "class A { #field = 1; #method() { return this.#field; } } class B { #field = 2; #method() { return this.#field; } }",
        "class A { #field = 1; #method() { return this.#field; } } class B { #field2 = 2; #method2() { return this.#field2; } }",
        "class Outer { #shared = 1; #getInner() { return class { #method() { return this.#shared; } }; } }",
    ];

    let mut snapshot = String::new();
    cases.into_iter().fold(&mut snapshot, |w, case| {
        let options = MangleOptions::default();
        write!(w, "{case}\n{}\n", mangle(case, options)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("private_member_mangling", snapshot);
    });
}

/// A named function expression whose name is shadowed by a same-named declaration in its
/// body must receive the same mangled name as the shadowing symbol; otherwise the emitted
/// fn-expr name collides with whichever unrelated outer-scope variable happens to own slot 0.
#[test]
fn function_expression_name_shadowed() {
    let options = MangleOptions::default();

    // `var` shadow.
    test(
        "function _() { var x; var f = function foo() { var foo = x; } }",
        "function _() { var e; var t = function t() { var t = e; } }",
        options,
    );

    // Parameter shadow.
    test(
        "function _() { var x; (function foo(foo) { foo + x })() }",
        "function _() { var e; (function t(t) { t + e; })(); }",
        options,
    );

    // `var` inside an `else` block — still hoists through the block scope to the fn-expr scope.
    test(
        "function _() { var x; var f = function foo() { if (x) {} else { var foo = x; } } }",
        "function _() { var e; var t = function t() { if (e) {} else { var t = e; } } }",
        options,
    );
}

/// Re-mangling a mangled output must be a fixed point.
#[test]
fn shadowed_fn_expr_mangle_is_idempotent() {
    let options = MangleOptions::default();

    let cases = [
        // Basic `var` shadow (sanity).
        "function _() { var x; var f = function foo() { var foo = x; } }",
        // Two shadowed fn-exprs in the same scope — unique coverage beyond `_shadowed` test.
        "
        (function() {
            var a = 1;
            var b = function foo() { var foo = a; };
            var c = function bar() { var bar = a; };
        })();
        ",
    ];

    for case in cases {
        let pass1 = mangle(case, options);
        let pass2 = mangle(&pass1, options);
        assert_eq!(
            pass1, pass2,
            "\nIdempotency failure for:\n{case}\nPass 1:\n{pass1}\nPass 2:\n{pass2}"
        );
    }
}

/// Annex B.3.2.1: In sloppy mode, function declarations inside blocks have var-like hoisting.
/// The mangler must not assign the same name to such a function and an outer `var` binding.
#[test]
fn annex_b_block_scoped_function() {
    let cases = [
        // Core bug: var + block function in if statement (vitejs/vite#22009)
        "function _() { var x = 1; if (true) { function y() {} } use(x); }",
        // var + block function in try block (oxc-project/oxc#14316)
        "function _() { var x = 1; try { function y() {} } finally {} use(x); }",
        // var + block function in plain block
        "function _() { var x = 1; { function y() {} } use(x); }",
        // Parameter + block function
        "function _(x) { if (true) { function y() {} } use(x); }",
        // Deeply nested blocks
        "function _() { var x = 1; { { if (true) { function y() {} } } } use(x); }",
        // Multiple block functions in same scope
        "function _() { var x = 1; if (true) { function y() {} function z() {} } use(x); }",
        // Block function referencing outer var
        "function _() { var x = 1; if (true) { function y() { return x; } } use(x); }",
        // Annex B function reuses name from sibling function scope (hoisting enables this)
        "function _() { function foo() { var x; use(x); } function bar() { if (true) { function baz() {} use(baz); } } }",
        // typeof must not be replaced with a constant (reviewer request)
        "console.log(typeof foo); if (true) { function foo() { return 1; } }",
    ];

    let mut snapshot = String::new();
    cases.into_iter().fold(&mut snapshot, |w, case| {
        let options = MangleOptions::default();
        write!(w, "{case}\n{}\n", mangle_script(case, options)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("annex_b_block_scoped_function", snapshot);
    });
}
