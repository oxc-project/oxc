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

fn mangle_jsx(source_text: &str, options: MangleOptions) -> String {
    mangle_with_source_type(source_text, options, SourceType::jsx().with_unambiguous(true))
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

/// Assert that every simple identifier JSX tag (opening and closing) in `mangled` starts with
/// an upper-case letter. Returns the list of tag names found (may contain duplicates).
fn assert_jsx_tags_upper_case(mangled: &str) -> Vec<String> {
    // split('<') yields [text_before_first_<, text_after_each_<, ...]; skip the first.
    let mut tags = Vec::new();
    for segment in mangled.split('<').skip(1) {
        let trimmed = segment.trim();
        if trimmed.starts_with('!') {
            continue;
        }
        let trimmed = trimmed.strip_prefix('/').unwrap_or(trimmed);
        let tag: String =
            trimmed.chars().take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '.').collect();
        if tag.is_empty() || tag.contains('.') {
            continue;
        }
        assert!(
            tag.starts_with(|c: char| c.is_ascii_uppercase()),
            "JSX component tag should start with upper-case, got: {tag}\nfull output: {mangled}"
        );
        tags.push(tag);
    }
    tags
}

#[test]
fn jsx_component_mangling() {
    let options = MangleOptions { top_level: Some(true), ..MangleOptions::default() };

    // Component tag names must start with upper-case after mangling
    let mangled =
        mangle_jsx("function MyComponent() { return null; } let x = <MyComponent />;", options);
    let tags = assert_jsx_tags_upper_case(&mangled);
    assert!(!tags.is_empty(), "Expected to find at least one JSX tag in: {mangled}");

    // Regular variables (not used as JSX tags) should still get lower-case-first names
    let mangled = mangle_jsx(
        "function MyComponent() { return null; } let regularVar = 1; let x = <MyComponent />;",
        options,
    );
    assert!(!mangled.contains("regularVar"), "regularVar should be mangled: {mangled}");

    // Member expressions don't need upper-case: <foo.bar /> is always a component
    let mangled =
        mangle_jsx("let foo = { bar: function() { return null; } }; let x = <foo.bar />;", options);
    assert!(
        mangled.contains(".bar"),
        "Member expression JSX should preserve member access: {mangled}"
    );

    // Component with closing tag: opening tag must be upper-case (closing tag follows automatically)
    let mangled =
        mangle_jsx("function Comp() { return null; } let x = <Comp>child</Comp>;", options);
    let tags = assert_jsx_tags_upper_case(&mangled);
    assert!(!tags.is_empty(), "Expected at least one JSX tag in: {mangled}");

    // Nested scope with default top_level (not Some(true)): symbols inside functions
    // are always mangled regardless of top_level setting
    let mangled = mangle_jsx(
        "function wrapper() { function Comp() { return null; } let x = <Comp />; }",
        MangleOptions::default(),
    );
    let tags = assert_jsx_tags_upper_case(&mangled);
    assert!(!tags.is_empty(), "Nested JSX component should be mangled: {mangled}");

    // Collision avoidance: create enough symbols that the regular pool claims both
    // 'S' and 'C' (the first two upper-case base54 names). The JSX fixup must skip
    // both and use 'T' (the next base54_upper_first name) instead.
    let mangled = mangle_jsx(
        "
        function Comp() { return null; }
        let v0, v1, v2, v3, v4, v5, v6, v7, v8, v9;
        let v10, v11, v12, v13, v14, v15, v16, v17, v18, v19;
        let v20, v21;
        console.log(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9);
        console.log(v10, v11, v12, v13, v14, v15, v16, v17, v18, v19);
        console.log(v20, v21);
        let x = <Comp />;
        ",
        options,
    );
    let tags = assert_jsx_tags_upper_case(&mangled);
    // With 24 slots total, the regular pool claims base54 indices 0-23, which includes
    // 'S' (index 21) and 'C' (index 22). The JSX fixup skips both and assigns 'T'.
    assert!(
        mangled.contains("let S") || mangled.contains(", S"),
        "Expected regular pool to claim 'S': {mangled}"
    );
    assert_eq!(tags, ["T"], "Expected JSX component to get 'T' (skipping 'S' and 'C'): {mangled}");

    // TSX (TypeScript + JSX) should also produce upper-case component names
    let tsx_mangled = mangle_with_source_type(
        "function Comp() { return null; } let x = <Comp />;",
        options,
        SourceType::tsx().with_unambiguous(true),
    );
    let tags = assert_jsx_tags_upper_case(&tsx_mangled);
    assert_eq!(tags, ["S"], "TSX component should be mangled to upper-case: {tsx_mangled}");
}

#[test]
fn jsx_component_mangling_debug_mode() {
    let options = MangleOptions { top_level: Some(true), debug: true, ..MangleOptions::default() };

    // In debug mode, JSX component symbols get "Slot_N" (capital S) via fixup.
    // All slots initially get "slot_N", then JSX slots are replaced.
    let mangled = mangle_jsx(
        "function Comp() { return null; } let regularVar = 1; let x = <Comp />;",
        options,
    );
    // Comp is a JSX component → initially slot_0, fixed up to Slot_0.
    // regularVar and x are regular → slot_1, slot_2.
    assert!(mangled.contains("Slot_0"), "JSX component should get Slot_0: {mangled}");
    assert!(mangled.contains("slot_1"), "Regular var should get slot_1: {mangled}");
}

#[test]
fn non_jsx_source_gets_full_base54_names() {
    // Non-JSX source types (.js, .mjs, .ts) should produce identical output regardless of
    // upper-case-named symbols in source.
    let options = MangleOptions { top_level: Some(true), ..MangleOptions::default() };
    let mangled = mangle("function Comp() { return null; } let regularVar = 1;", options);
    assert!(
        mangled.starts_with("function e("),
        "Non-JSX source should mangle with names from full base54 set: {mangled}"
    );
}

#[test]
fn jsx_component_mangling_respects_keep_names() {
    // A component whose function name is preserved by keep_names should not enter the JSX pool
    // (it's filtered before slot assignment and keeps its original name).
    let options = MangleOptions {
        top_level: Some(true),
        keep_names: MangleOptionsKeepNames::all_true(),
        ..MangleOptions::default()
    };
    let mangled = mangle_jsx(
        "function Comp() { return null; } let regularVar = 1; let x = <Comp />;",
        options,
    );
    // With keep_names, function name "Comp" is preserved
    assert!(
        mangled.contains("function Comp("),
        "keep_names should preserve function name: {mangled}"
    );
    // The JSX tag should still use the preserved name
    assert!(mangled.contains("<Comp"), "JSX tag should use preserved name: {mangled}");
}

#[test]
fn jsx_many_components() {
    // With >26 JSX components, multi-character base54_upper_first names are generated
    // (e.g. "Se", "Ce"). All must still start with upper-case.
    let options = MangleOptions { top_level: Some(true), ..MangleOptions::default() };

    // Generate 30 component functions + JSX usage
    let mut source = String::new();
    for i in 0..30 {
        write!(source, "function Comp{i}() {{ return null; }} ").unwrap();
    }
    for i in 0..30 {
        write!(source, "let x{i} = <Comp{i} />; ").unwrap();
    }

    let mangled = mangle_jsx(&source, options);
    let tags = assert_jsx_tags_upper_case(&mangled);
    assert!(tags.len() >= 30, "Expected at least 30 JSX tags, got {}: {mangled}", tags.len());
    // With 30 components, some names must be 2+ characters (only 26 single-char upper-case names)
    assert!(
        tags.iter().any(|t| t.len() > 1),
        "Expected some multi-character upper-case names with 30 components: {mangled}"
    );
}

#[test]
fn jsx_component_in_eval_scope() {
    // A component declared in a scope with eval() should keep its original name
    // and not enter any mangling pool.
    let options = MangleOptions::default();
    let mangled = mangle_jsx(
        "function wrapper() { function Comp() { return null; } let x = <Comp />; eval(''); }",
        options,
    );
    // The eval scope keeps original names
    assert!(
        mangled.contains("function Comp("),
        "Component in eval scope should keep its name: {mangled}"
    );
    assert!(
        mangled.contains("<Comp"),
        "JSX tag in eval scope should keep original name: {mangled}"
    );
}

#[test]
fn jsx_slot_sharing_between_jsx_and_regular() {
    // When a JSX component and a regular variable are in sibling (non-conflicting) scopes,
    // the slot assignment can place them in the same slot. Since one symbol in the slot is
    // a JSX component, the whole slot goes to the JSX pool, giving the regular variable an
    // upper-case name too. This is correct (upper-case is always a valid JS identifier).
    let options = MangleOptions::default();
    let mangled = mangle_jsx(
        "function outer() { { function Comp() { return null; } let x = <Comp />; } { let y = 1; console.log(y); } }",
        options,
    );
    // Both Comp and y land in the same slot (sibling scopes) and that slot is JSX,
    // so both get the upper-case name "S". x is in a separate slot → "e".
    let tags = assert_jsx_tags_upper_case(&mangled);
    assert_eq!(tags, ["S"], "Expected exactly one JSX tag 'S': {mangled}");
    assert!(mangled.contains("function S("), "Comp should be mangled to S: {mangled}");
    assert!(mangled.contains("let S = 1"), "y should share slot with Comp and become S: {mangled}");
}

#[test]
fn jsx_component_declared_but_never_used_as_tag() {
    // A function with an upper-case name that is never used in a JSX tag position should
    // stay in the regular pool and can get a lower-case name.
    let options = MangleOptions { top_level: Some(true), ..MangleOptions::default() };
    let mangled = mangle_jsx("function Comp() { return null; } let x = Comp;", options);
    // Comp is never in a <Comp /> position, so it's a regular symbol
    assert!(!mangled.contains("Comp"), "Comp should be mangled: {mangled}");
    // It should get a lower-case-first name from the regular pool
    assert!(
        mangled.starts_with("function e("),
        "Unused-as-tag component should get a regular (lower-case-first) name: {mangled}"
    );
}

#[test]
fn jsx_intrinsic_html_tags_unchanged() {
    // Intrinsic HTML tags like <div>, <br />, <h1> are not identifiers bound to symbols.
    // They must pass through the mangler unchanged and remain lower-case.
    let options = MangleOptions { top_level: Some(true), ..MangleOptions::default() };

    let mangled = mangle_jsx(
        r"function Comp() { return <div><h1>hello</h1><br /><span>world</span></div>; } let x = <Comp />;",
        options,
    );
    // HTML tags must survive verbatim
    for tag in ["div", "h1", "br", "span"] {
        assert!(
            mangled.contains(&format!("<{tag}")),
            "Intrinsic HTML tag <{tag}> should be preserved: {mangled}"
        );
    }
    // The component must be mangled to an upper-case name (not one of the HTML tags)
    assert!(
        mangled.contains("<S ") || mangled.contains("<S>"),
        "Component should be mangled to upper-case name: {mangled}"
    );
}

#[test]
fn jsx_mangler() {
    let options = MangleOptions { top_level: Some(true), ..MangleOptions::default() };

    let cases = [
        // Basic component
        "function Comp() { return null; } let x = <Comp />;",
        // Component mixed with regular vars
        "function Comp() { return null; } let a = 1; let b = 2; let x = <Comp />;",
        // Member expression (foo doesn't need upper-case)
        "let ns = { Bar: function() { return null; } }; let x = <ns.Bar />;",
        // Multiple components
        "function A() { return null; } function B() { return null; } let x = <A />; let y = <B />;",
        // Component also used as a regular value
        "function Comp() { return null; } let x = <Comp />; let y = Comp;",
        // Component with closing tag
        "function Comp() { return null; } let x = <Comp>child</Comp>;",
    ];

    let mut snapshot = String::new();
    cases.into_iter().fold(&mut snapshot, |w, case| {
        write!(w, "{case}\n{}\n", mangle_jsx(case, options)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("jsx_mangler", snapshot);
    });
}
