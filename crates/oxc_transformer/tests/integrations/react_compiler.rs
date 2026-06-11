use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_react_compiler::default_plugin_options;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_transformer::{TransformOptions, Transformer};

#[test]
fn memoizes_component_through_transformer() {
    let allocator = Allocator::default();
    let source = "export function Greeting({ name }) {\n  return <div>Hello {name}</div>;\n}\n";
    let mut program = Parser::new(&allocator, source, SourceType::jsx()).parse().program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

    let options =
        TransformOptions { react_compiler: Some(default_plugin_options()), ..Default::default() };
    let ret = Transformer::new(&allocator, Path::new("greeting.jsx"), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.is_empty(), "{:?}", ret.diagnostics);

    let code = Codegen::new().build(&program).code;
    assert!(code.contains("react/compiler-runtime"), "missing runtime import:\n{code}");
    assert!(code.contains("_c("), "component was not memoized:\n{code}");
}
