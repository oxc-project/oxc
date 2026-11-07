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

#[test]
fn folded_enum_initializers_remove_const_references() {
    for optimize_enums in [false, true] {
        let allocator = Allocator::default();
        let mut program =
            Parser::new(&allocator, r#"const o = "o"; enum E { A = o }"#, SourceType::ts())
                .parse()
                .program;
        let scoping =
            SemanticBuilder::new().with_enum_eval(true).build(&program).semantic.into_scoping();
        let symbol_id = scoping.symbol_ids().find(|&id| scoping.symbol_name(id) == "o").unwrap();
        assert_eq!(scoping.get_resolved_reference_ids(symbol_id).len(), 1);
        let mut options = TransformOptions::default();
        options.typescript.optimize_enums = optimize_enums;
        let ret = Transformer::new(&allocator, Path::new("a.ts"), &options)
            .build_with_scoping(scoping, &mut program);
        assert!(ret.diagnostics.is_empty());
        assert_eq!(ret.scoping.get_resolved_reference_ids(symbol_id).len(), 0);
    }
}
