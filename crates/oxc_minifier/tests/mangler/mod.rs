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
    let mangler =
        Mangler::new().with_options(MangleOptions { debug: false, top_level }).build(&program);
    CodeGenerator::new().with_mangler(Some(mangler)).build(&program).code
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
