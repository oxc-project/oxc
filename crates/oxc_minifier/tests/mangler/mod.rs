use std::fmt::Write;

use oxc_allocator::Allocator;
use oxc_codegen::CodeGenerator;
use oxc_mangler::Mangler;
use oxc_parser::Parser;
use oxc_span::SourceType;

fn mangle(source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::mjs();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let program = ret.program;
    let mangler = Mangler::new().build(&program);
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

    let snapshot = cases.into_iter().fold(String::new(), |mut w, case| {
        write!(w, "{case}\n{}\n", mangle(case)).unwrap();
        w
    });

    insta::with_settings!({ prepend_module_to_snapshot => false, omit_expression => true }, {
        insta::assert_snapshot!("mangler", snapshot);
    });
}
