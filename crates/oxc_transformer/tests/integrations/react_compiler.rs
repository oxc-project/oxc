use std::path::Path;

use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_react_compiler::{PanicThreshold, PluginOptions};
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

#[test]
fn nonfatal_bailout_does_not_discard_compilable_sibling() {
    let allocator = Allocator::default();
    let source =
        include_str!("../fixtures/react_compiler/nonfatal-bailout-with-compilable-sibling.jsx");
    let mut program = Parser::new(&allocator, source, SourceType::jsx()).parse().program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let options =
        TransformOptions { react_compiler: Some(PluginOptions::default()), ..Default::default() };

    let ret = Transformer::new(&allocator, Path::new("mixed.jsx"), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.has_errors(), "expected the ref-access bailout diagnostic");

    let code = Codegen::new().build(&program).code;
    assert!(code.contains("ref.current"), "bailed-out function was not preserved:\n{code}");
    assert!(code.contains("react/compiler-runtime"), "sibling was not compiled:\n{code}");
    assert!(code.contains("_c("), "sibling did not use the memo cache:\n{code}");
}

#[test]
fn nonfatal_bailout_allows_downstream_tsx_transform() {
    let allocator = Allocator::default();
    let source = include_str!("../fixtures/react_compiler/nonfatal-bailout-continues-tsx.tsx");
    let mut program = Parser::new(&allocator, source, SourceType::tsx()).parse().program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let options =
        TransformOptions { react_compiler: Some(PluginOptions::default()), ..Default::default() };

    let ret = Transformer::new(&allocator, Path::new("mixed.tsx"), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.has_errors(), "expected the ref-access bailout diagnostic");

    let code = Codegen::new().build(&program).code;
    assert!(code.contains("react/compiler-runtime"), "sibling was not compiled:\n{code}");
    assert!(!code.contains("type Props"), "TypeScript transform did not run:\n{code}");
    assert!(!code.contains(": JSX.Element"), "return type was not removed:\n{code}");
    assert!(!code.contains("<main>"), "JSX transform did not run:\n{code}");
}

#[test]
fn all_errors_panic_threshold_still_aborts_downstream_transforms() {
    let allocator = Allocator::default();
    let source = include_str!("../fixtures/react_compiler/nonfatal-bailout-continues-tsx.tsx");
    let mut program = Parser::new(&allocator, source, SourceType::tsx()).parse().program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let react_compiler =
        PluginOptions { panic_threshold: PanicThreshold::AllErrors, ..PluginOptions::default() };
    let options = TransformOptions { react_compiler: Some(react_compiler), ..Default::default() };

    let ret = Transformer::new(&allocator, Path::new("mixed.tsx"), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(ret.diagnostics.has_errors(), "expected a fatal ref-access diagnostic");

    let code = Codegen::new().build(&program).code;
    assert!(code.contains("type Props"), "fatal compile unexpectedly ran TypeScript transform");
    assert!(code.contains("<main>"), "fatal compile unexpectedly ran JSX transform");
    assert!(!code.contains("react/compiler-runtime"), "fatal compile emitted compiler output");
}

#[test]
fn config_error_still_aborts_downstream_transforms() {
    let allocator = Allocator::default();
    let source = include_str!("../fixtures/react_compiler/fatal-blocklisted-import.tsx");
    let mut program = Parser::new(&allocator, source, SourceType::tsx()).parse().program;
    let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
    let mut react_compiler = PluginOptions::default();
    react_compiler.environment.validate_blocklisted_imports =
        Some(vec!["blocked-module".to_string()]);
    let options = TransformOptions { react_compiler: Some(react_compiler), ..Default::default() };

    let ret = Transformer::new(&allocator, Path::new("blocked.tsx"), &options)
        .build_with_scoping(scoping, &mut program);
    assert!(!ret.diagnostics.is_empty(), "expected a fatal configuration diagnostic");

    let code = Codegen::new().build(&program).code;
    assert!(code.contains(": JSX.Element"), "config error unexpectedly ran TypeScript transform");
    assert!(code.contains("<div>"), "config error unexpectedly ran JSX transform");
}
