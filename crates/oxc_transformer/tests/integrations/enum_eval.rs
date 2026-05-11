use std::path::Path;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};

fn run(enum_eval: bool) {
    let source_text = "enum E { A = 1 }";
    let source_type = SourceType::ts();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let mut program = ret.program;
    let scoping =
        SemanticBuilder::new().with_enum_eval(enum_eval).build(&program).semantic.into_scoping();
    let _ret = Transformer::new(&allocator, Path::new(""), &TransformOptions::default())
        .build_with_scoping(scoping, &mut program);
}

#[test]
fn transforms_enum_when_enum_eval_enabled() {
    run(true);
}

#[test]
#[should_panic(expected = "SemanticBuilder::with_enum_eval(true)")]
#[cfg(debug_assertions)]
fn panics_when_enum_eval_disabled() {
    run(false);
}
