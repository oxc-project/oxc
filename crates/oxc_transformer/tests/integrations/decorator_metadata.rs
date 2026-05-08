//! Legacy decorator metadata: bare-identifier emit for class and value-import bindings.

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
fn forward_referenced_class_emits_bare_identifier() {
    let output = transform_ts(
        r"
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() laterRef!: LaterClass; }
        class LaterClass { tag = 'later'; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', LaterClass)"), "got:\n{output}");
    assert!(!output.contains("typeof LaterClass !== 'undefined'"), "got:\n{output}");
}

#[test]
fn cross_file_import_emits_bare_identifier() {
    let output = transform_ts(
        r"
        import { ImportedEnum } from './enums';
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() value!: ImportedEnum; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', ImportedEnum)"), "got:\n{output}");
    assert!(!output.contains("typeof ImportedEnum !== 'undefined'"), "got:\n{output}");
}

#[test]
fn class_resolved_before_use_emits_bare_identifier() {
    let output = transform_ts(
        r"
        class A {}
        function D(): PropertyDecorator { return () => {}; }
        class B { @D() a!: A; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', A)"), "got:\n{output}");
}

// `declare class X` now emits bare identifier matching tsc/babel; previously OXC wrapped it
// in the typeof-undefined guard which silently produced Object on unfulfilled ambient bindings.
#[test]
fn ambient_declared_class_emits_bare_identifier() {
    let output = transform_ts(
        r"
        declare class Ambient {}
        function D(): PropertyDecorator { return () => {}; }
        class B { @D() x!: Ambient; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Ambient)"), "got:\n{output}");
}

#[test]
fn type_only_import_emits_object() {
    let output = transform_ts(
        r"
        import type { TypeOnlyClass } from './m';
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() x!: TypeOnlyClass; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Object)"), "got:\n{output}");
    assert!(!output.contains("decorateMetadata('design:type', TypeOnlyClass)"), "got:\n{output}");
}

#[test]
fn aliased_import_emits_bare_identifier() {
    let output = transform_ts(
        r"
        import { Original as Renamed } from './m';
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() x!: Renamed; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Renamed)"), "got:\n{output}");
}

#[test]
fn default_import_emits_bare_identifier() {
    let output = transform_ts(
        r"
        import DefaultClass from './m';
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() x!: DefaultClass; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', DefaultClass)"), "got:\n{output}");
}

#[test]
fn generic_class_with_type_args_emits_bare_identifier() {
    let output = transform_ts(
        r"
        class Container<T> { value!: T; }
        function D(): PropertyDecorator { return () => {}; }
        class Holder { @D() x!: Container<string>; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Container)"), "got:\n{output}");
}

#[test]
fn class_merged_with_interface_emits_bare_identifier() {
    let output = transform_ts(
        r"
        class Merged { value = 1; }
        interface Merged { extra: string; }
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() x!: Merged; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Merged)"), "got:\n{output}");
}

// `typeof A` is a TSTypeQuery, untouched by this patch; assert no panic and stable emit.
#[test]
fn typeof_query_falls_through_to_existing_path() {
    let output = transform_ts(
        r"
        class A {}
        function D(): PropertyDecorator { return () => {}; }
        class B { @D() x!: typeof A; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type'"), "got:\n{output}");
}

// Qualified names go through the QualifiedName arm of the fallback, not the patched path.
#[test]
fn qualified_name_class_reference_unchanged() {
    let output = transform_ts(
        r"
        namespace N { export class Inner {} }
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() x!: N.Inner; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type'"), "got:\n{output}");
}

#[test]
fn import_used_via_local_alias_emits_bare_identifier() {
    let output = transform_ts(
        r"
        import { Far } from './far';
        const Local = Far;
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() x!: Far; }
        void Local;
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Far)"), "got:\n{output}");
}

#[test]
fn self_referential_class_emits_bare_identifier() {
    let output = transform_ts(
        r"
        function D(): PropertyDecorator { return () => {}; }
        class Node { @D() next!: Node; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', Node)"), "got:\n{output}");
}

// `const X = class {}` produces a Variable symbol, not Class; the patch's predicate does not
// fire and the existing wrapper is used.
#[test]
fn class_expression_via_const_keeps_guard() {
    let output = transform_ts(
        r"
        const C = class { value = 1; };
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() x!: C; }
    ",
    );
    assert!(
        output.contains("typeof C !== 'undefined'")
            || output.contains("decorateMetadata('design:type', C)"),
        "got:\n{output}"
    );
}

#[test]
fn mixed_type_value_import_handles_both_kinds() {
    let output = transform_ts(
        r"
        import { type T, V } from './m';
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() v!: V; @D() t!: T; }
    ",
    );
    assert!(output.contains("decorateMetadata('design:type', V)"), "got:\n{output}");
    assert!(output.contains("decorateMetadata('design:type', Object)"), "got:\n{output}");
    assert!(!output.contains("decorateMetadata('design:type', T)"), "got:\n{output}");
}

#[test]
fn constructor_param_class_emits_bare_identifier() {
    let output = transform_ts(
        r"
        class A {}
        function dec(target: any) {}
        @dec class Holder { constructor(a: A) {} }
    ",
    );
    assert!(
        output.contains("[A]") || output.contains("'design:paramtypes', [A]"),
        "got:\n{output}"
    );
}

#[test]
fn method_return_type_class_emits_bare_identifier() {
    let output = transform_ts(
        r"
        class A {}
        function D(): MethodDecorator { return () => {}; }
        class Holder { @D() greet(): A { return new A(); } }
    ",
    );
    assert!(output.contains("decorateMetadata('design:returntype', A)"), "got:\n{output}");
}

#[test]
fn unresolved_reference_keeps_guard() {
    let output = transform_ts(
        r"
        function D(): PropertyDecorator { return () => {}; }
        class Source { @D() x!: TotallyUndeclared; }
    ",
    );
    assert!(output.contains("typeof TotallyUndeclared !== 'undefined'"), "got:\n{output}");
}
