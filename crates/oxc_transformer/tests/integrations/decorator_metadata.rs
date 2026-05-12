use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{DecoratorOptions, TransformOptions, Transformer};

fn transform_ts(source_text: &str) -> String {
    let allocator = Allocator::default();
    let source_type = SourceType::ts();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.errors.is_empty(), "parse errors: {:?}", ret.errors);
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let options = TransformOptions {
        decorator: DecoratorOptions { legacy: true, emit_decorator_metadata: true },
        ..TransformOptions::default()
    };
    let ret = Transformer::new(&allocator, Path::new(""), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.errors.is_empty(), "transform errors: {:?}", ret.errors);
    Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code
}

const PREFIX: &str = "function D(): PropertyDecorator { return () => {}; } ";

#[test]
fn nullable_string_emits_string() {
    let src = format!("{PREFIX} class Source {{ @D() value!: string | null; }}");
    let output = transform_ts(&src);
    assert!(output.contains("decorateMetadata('design:type', String)"), "got:\n{output}");
}

#[test]
fn undefined_union_emits_underlying_primitive() {
    let src = format!("{PREFIX} class Source {{ @D() value!: number | undefined; }}");
    let output = transform_ts(&src);
    assert!(output.contains("decorateMetadata('design:type', Number)"), "got:\n{output}");
}

#[test]
fn nullable_and_undefined_emits_underlying_primitive() {
    let src = format!("{PREFIX} class Source {{ @D() value!: boolean | null | undefined; }}");
    let output = transform_ts(&src);
    assert!(output.contains("decorateMetadata('design:type', Boolean)"), "got:\n{output}");
}

#[test]
fn null_only_union_still_emits_void() {
    let src = format!("{PREFIX} class Source {{ @D() value!: null | undefined; }}");
    let output = transform_ts(&src);
    assert!(output.contains("decorateMetadata('design:type', void 0)"), "got:\n{output}");
}

#[test]
fn distinct_primitive_union_still_emits_object() {
    let src = format!("{PREFIX} class Source {{ @D() value!: string | number; }}");
    let output = transform_ts(&src);
    assert!(output.contains("decorateMetadata('design:type', Object)"), "got:\n{output}");
}

#[test]
fn intersection_with_null_emits_object() {
    let src = format!("{PREFIX} class Source {{ @D() value!: string & null; }}");
    let output = transform_ts(&src);
    assert!(output.contains("decorateMetadata('design:type', Object)"), "got:\n{output}");
}
