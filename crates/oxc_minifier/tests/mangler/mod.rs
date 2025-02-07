use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_mangler::{MangleOptions, Mangler};
use oxc_parser::Parser;
use oxc_span::SourceType;

fn mangle(source_text: &str, top_level: bool) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = ret.program;
    let symbol_table =
        Mangler::new().with_options(MangleOptions { debug: false, top_level }).build(&program);
    CodeGenerator::new().with_symbol_table(Some(symbol_table)).build(&program).code
}

#[test]
fn mangler() {
    let cases = [
        "function foo(a) {a}",
        "function foo(a) { let _ = { x } }",
        "function foo(a) { let { x } = y }",
        "var x; function foo(a) { ({ x } = y) }",
        "import { x } from 's'; export { x }",
        "function _ (exports) { Object.defineProperty(exports, '__esModule', { value: true }) }",
        "function foo(foo_a, foo_b, foo_c) {}; function bar(bar_a, bar_b, bar_c) {}", // foo_a and bar_a can be reused
        "function _() { function foo() { var x; foo; } }", // x should not use the same name with foo
        "function _() { var x; function foo() { var y; function bar() { x } } }", // y should not shadow x
        "function _() { function x(a) {} }",                                      // a can shadow x
        "function _() { function x(a) { x } }", // a should not shadow x
        "function _() { var x; { var y }}",     // y should not shadow x
        "function _() { var x; { let y }}",     // y can shadow x
        "function _() { let x; { let y }}",     // y can shadow x
        "function _() { var x; { const y }}",   // y can shadow x
        "function _() { let x; { const y }}",   // y can shadow x
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
    ];
    let top_level_cases = [
        "function foo(a) {a}",
        "export function foo() {}; foo()",
        "export default function foo() {}; foo()",
        "export const foo = 1; foo",
        "const foo = 1; foo; export { foo }",
    ];

    let mut snapshot = String::new();
    cases.into_iter().fold(&mut snapshot, |w, case| {
        write!(w, "{case}\n{}\n", mangle(case, false)).unwrap();
        w
    });
    top_level_cases.into_iter().fold(&mut snapshot, |w, case| {
        write!(w, "{case}\n{}\n", mangle(case, true)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("mangler", snapshot);
    });
}
