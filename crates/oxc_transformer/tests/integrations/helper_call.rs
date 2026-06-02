use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::test;

fn codegen_options() -> CodegenOptions {
    CodegenOptions { single_quote: true, ..CodegenOptions::default() }
}

/// Verify that transform → codegen is idempotent (the second codegen pass produces the same output).
/// Helper calls like `_defineProperty` must not pick up unrelated comments from the original source.
#[test]
fn helper_call_idempotency() {
    use std::str::FromStr;
    let options = oxc_transformer::TransformOptions::from(
        oxc_transformer::ESTarget::from_str("es2015").unwrap(),
    );

    let cases = [
        // `_defineProperty` — comment inside the value argument should not break formatting
        "class C { prop = new Foo(1 /* type */); }",
        // `_classPrivateFieldSet` — comment inside nested call
        "class C { #x; constructor() { this.#x = new Foo(1 /* type */); } }",
        // `_defineProperty` — comment inside function body argument
        "class C { prop = function() { /* comment */ return 1; }; }",
        // `_defineProperty` — trailing comment after last argument
        "class C { prop = foo(a, b /* last */); }",
    ];

    for source in cases {
        let first = test(source, &options).expect("transform should succeed");

        let allocator = Allocator::default();
        let source_type = SourceType::default();
        let ret = Parser::new(&allocator, &first, source_type).parse();
        assert!(ret.errors.is_empty(), "Parse errors on second pass for: {source}");
        let second = Codegen::new().with_options(codegen_options()).build(&ret.program).code;

        assert_eq!(
            first, second,
            "Codegen not idempotent after transform.\nInput: {source}\nFirst:  {first}\nSecond: {second}"
        );
    }
}
