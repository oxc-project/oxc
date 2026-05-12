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

#[test]
fn readonly_array_emits_array() {
    let output = transform_ts(
        r"
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() value!: ReadonlyArray<string>; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Array)"), "got:\n{output}");
}

#[test]
fn nested_readonly_array_emits_array() {
    let output = transform_ts(
        r"
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() value!: ReadonlyArray<ReadonlyArray<number>>; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Array)"), "got:\n{output}");
}

#[test]
fn readonly_short_form_still_emits_array() {
    let output = transform_ts(
        r"
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() value!: readonly string[]; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Array)"), "got:\n{output}");
}

#[test]
fn user_shadowed_readonly_array_class_takes_precedence() {
    let output = transform_ts(
        r"
        class ReadonlyArray<T> { value!: T; }
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() value!: ReadonlyArray<string>; }
    ",
    );
    assert!(
        !output.contains("decorateMetadata('design:type', Array)"),
        "user-shadowed ReadonlyArray must not be miscompiled to Array. got:\n{output}"
    );
}
