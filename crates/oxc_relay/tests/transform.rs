use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_relay::{Relay, RelayLanguage, RelayOptions};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

fn transform(source_path: &str, source_text: &str, options: RelayOptions) -> (String, bool) {
    let source_type = SourceType::from_path(Path::new(source_path)).unwrap();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.diagnostics.is_empty(), "parse errors for source {source_text}");
    let mut program = ret.program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let ret = Relay::new(options, Path::new(source_path)).build(&allocator, &mut program, scoping);
    let code = Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&program)
        .code;
    (code, ret.diagnostics.has_errors())
}

fn codegen(source_path: &str, source_text: &str) -> String {
    let source_type = SourceType::from_path(Path::new(source_path)).unwrap();
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    assert!(ret.diagnostics.is_empty(), "parse errors for expected {source_text}");
    Codegen::new()
        .with_options(CodegenOptions { single_quote: true, ..CodegenOptions::default() })
        .build(&ret.program)
        .code
}

#[track_caller]
fn test_with_path(source_path: &str, source_text: &str, options: RelayOptions, expected: &str) {
    let (code, has_errors) = transform(source_path, source_text, options);
    assert!(!has_errors, "unexpected diagnostics for source {source_text}");
    assert_eq!(code, codegen(source_path, expected), "for source {source_text}");
}

#[test]
fn transforms_graphql_in_tsx() {
    test_with_path(
        "component.tsx",
        "interface Props { id: string }
        const style = css`color: red`;
        const data = graphql`query FooQuery { id }`;
        export const App = (props: Props) => <div>{data}</div>;",
        RelayOptions::default(),
        "import _FooQuery from './__generated__/FooQuery.graphql.js';
        interface Props { id: string }
        const style = css`color: red`;
        const data = _FooQuery;
        export const App = (props: Props) => <div>{data}</div>;",
    );
}

#[test]
fn supports_relay_options() {
    test_with_path(
        "project/src/pages/foo.ts",
        "const data = graphql`fragment Foo_item on Item { id }`;",
        RelayOptions {
            artifact_directory: Some("project/src/__generated__".into()),
            language: RelayLanguage::Typescript,
            eager_es_modules: false,
        },
        "const data = require('../__generated__/Foo_item.graphql.ts');",
    );
}

#[test]
fn reports_unnamed_documents() {
    let source = "const data = graphql`{ id }`;";
    let (code, has_errors) = transform("test.js", source, RelayOptions::default());
    assert!(has_errors);
    assert_eq!(code, codegen("test.js", source));
}
