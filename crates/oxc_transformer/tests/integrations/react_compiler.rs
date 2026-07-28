use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_react_compiler::PluginOptions;
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
        TransformOptions { react_compiler: Some(PluginOptions::default()), ..Default::default() };
    let ret = Transformer::new(&allocator, Path::new("greeting.jsx"), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.is_empty(), "{:?}", ret.diagnostics);

    let code = Codegen::new().build(&program).code;
    assert!(code.contains("react/compiler-runtime"), "missing runtime import:\n{code}");
    assert!(code.contains("_c("), "component was not memoized:\n{code}");
}

/// An import referenced only through a computed key (`{ [NAME]: … }`) inside a
/// memoized block must survive. The compiler hoists the object into a cache slot;
/// the computed key has to stay an identifier *reference* so semantic analysis
/// links it to the import and TypeScript import elision keeps the import alive —
/// otherwise the emitted `[NAME]` dangles. Uses `.tsx` so import elision runs.
#[test]
fn keeps_import_used_only_as_computed_key() {
    let allocator = Allocator::default();
    let source = "import { CSS_VAR } from './styles.css';\n\
        export function Box({ size }) {\n  \
            const style = { [CSS_VAR]: size + 'px' };\n  \
            return <div style={style} />;\n}\n";
    let mut program = Parser::new(&allocator, source, SourceType::tsx()).parse().program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

    let options =
        TransformOptions { react_compiler: Some(PluginOptions::default()), ..Default::default() };
    let ret = Transformer::new(&allocator, Path::new("box.tsx"), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.is_empty(), "{:?}", ret.diagnostics);

    let code = Codegen::new().build(&program).code;
    assert!(code.contains("_c("), "component was not memoized:\n{code}");
    assert!(code.contains("[CSS_VAR]"), "computed key was lost:\n{code}");
    assert!(
        code.contains("import { CSS_VAR }"),
        "import referenced by the computed key was wrongly elided:\n{code}"
    );
}
