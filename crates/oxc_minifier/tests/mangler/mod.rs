use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_mangler::{MangleOptions, MangleOptionsKeepNames, Mangler};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn mangle(source_text: &str, options: MangleOptions) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
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

#[test]
fn direct_eval() {
    let source_text = "function foo() { let NO_MANGLE; eval('') }";
    let options = MangleOptions::default();
    let mangled = mangle(source_text, options);
    assert_eq!(mangled, "function foo() {\n\tlet NO_MANGLE;\n\teval(\"\");\n}\n");
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
        let options = MangleOptions { top_level: true, ..MangleOptions::default() };
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
