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
        TransformOptions { react_compiler: Some(default_plugin_options()), ..Default::default() };
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

/// A Rules-of-Hooks violation is non-fatal with the default `panicThreshold: 'none'`
/// (matching babel-plugin-react-compiler): the compiler skips the offending
/// function, so the transform still emits code and surfaces the issue as a
/// warning rather than aborting with an error and empty output.
#[test]
fn react_compiler_error_is_non_fatal_and_still_emits_code() {
    let allocator = Allocator::default();
    let source = "function Component(props) {\n  \
        if (props.cond) {\n    useState(0);\n  }\n  \
        return <div>{props.text}</div>;\n}\n";
    let mut program = Parser::new(&allocator, source, SourceType::jsx()).parse().program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();

    let options =
        TransformOptions { react_compiler: Some(default_plugin_options()), ..Default::default() };
    let ret = Transformer::new(&allocator, Path::new("component.jsx"), &options)
        .build_with_scoping(scoping, &mut program);

    // The compiler reported the violation, but not as a fatal error.
    assert!(!ret.diagnostics.is_empty(), "expected a diagnostic for the hook violation");
    assert!(!ret.diagnostics.has_errors(), "react compiler error should be demoted to a warning");

    // Code is still emitted (the original function, left uncompiled).
    let code = Codegen::new().build(&program).code;
    assert!(code.contains("function Component"), "original code should still be emitted:\n{code}");
}
