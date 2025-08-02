use std::process::Command;

use crate::{
    AppArgs,
    babel::{BabelCase, BabelSuite},
    misc::{MiscCase, MiscSuite},
    runtime::Test262RuntimeCase,
    suite::Suite,
    test262::{Test262Case, Test262Suite},
    tools::{
        codegen::{CodegenBabelCase, CodegenMiscCase, CodegenTest262Case, CodegenTypeScriptCase},
        estree::{AcornJsxSuite, EstreeJsxCase, EstreeTest262Case, EstreeTypescriptCase},
        formatter::{
            FormatterBabelCase, FormatterMiscCase, FormatterTest262Case, FormatterTypeScriptCase,
        },
        minifier::{MinifierBabelCase, MinifierTest262Case},
        semantic::{
            SemanticBabelCase, SemanticMiscCase, SemanticTest262Case, SemanticTypeScriptCase,
        },
        transformer::{
            TransformerBabelCase, TransformerMiscCase, TransformerTest262Case,
            TransformerTypeScriptCase,
        },
    },
    typescript::{TranspileRunner, TypeScriptCase, TypeScriptSuite, TypeScriptTranspileCase},
    workspace_root,
};

/// Parser task runner
pub fn run_parser(args: &AppArgs) {
    Test262Suite::<Test262Case>::new().run("parser_test262", args);
    BabelSuite::<BabelCase>::new().run("parser_babel", args);
    TypeScriptSuite::<TypeScriptCase>::new().run("parser_typescript", args);
    MiscSuite::<MiscCase>::new().run("parser_misc", args);
}

/// Semantic task runner
pub fn run_semantic(args: &AppArgs) {
    Test262Suite::<SemanticTest262Case>::new().run("semantic_test262", args);
    BabelSuite::<SemanticBabelCase>::new().run("semantic_babel", args);
    TypeScriptSuite::<SemanticTypeScriptCase>::new().run("semantic_typescript", args);
    MiscSuite::<SemanticMiscCase>::new().run("semantic_misc", args);
}

/// Codegen task runner
pub fn run_codegen(args: &AppArgs) {
    Test262Suite::<CodegenTest262Case>::new().run("codegen_test262", args);
    BabelSuite::<CodegenBabelCase>::new().run("codegen_babel", args);
    TypeScriptSuite::<CodegenTypeScriptCase>::new().run("codegen_typescript", args);
    MiscSuite::<CodegenMiscCase>::new().run("codegen_misc", args);
}

/// Formatter task runner
pub fn run_formatter(args: &AppArgs) {
    Test262Suite::<FormatterTest262Case>::new().run("formatter_test262", args);
    BabelSuite::<FormatterBabelCase>::new().run("formatter_babel", args);
    TypeScriptSuite::<FormatterTypeScriptCase>::new().run("formatter_typescript", args);
    MiscSuite::<FormatterMiscCase>::new().run("formatter_misc", args);
}

/// Transformer task runner
pub fn run_transformer(args: &AppArgs) {
    Test262Suite::<TransformerTest262Case>::new().run("transformer_test262", args);
    BabelSuite::<TransformerBabelCase>::new().run("transformer_babel", args);
    TypeScriptSuite::<TransformerTypeScriptCase>::new().run("transformer_typescript", args);
    MiscSuite::<TransformerMiscCase>::new().run("transformer_misc", args);
}

/// Transpiler task runner
pub fn run_transpiler(args: &AppArgs) {
    TranspileRunner::<TypeScriptTranspileCase>::new().run("transpile", args);
}

/// ESTree task runner
pub fn run_estree(args: &AppArgs) {
    Test262Suite::<EstreeTest262Case>::new().run("estree_test262", args);
    AcornJsxSuite::<EstreeJsxCase>::new().run("estree_acorn_jsx", args);
    TypeScriptSuite::<EstreeTypescriptCase>::new().run("estree_typescript", args);
}

/// Minifier task runner
pub fn run_minifier(args: &AppArgs) {
    Test262Suite::<MinifierTest262Case>::new().run("minifier_test262", args);
    BabelSuite::<MinifierBabelCase>::new().run("minifier_babel", args);
}

/// Runtime task runner
/// # Panics
pub fn run_runtime(args: &AppArgs) {
    let path = workspace_root().join("src/runtime/runtime.js").to_string_lossy().to_string();
    let mut runtime_process = Command::new("node")
        .args(["--experimental-vm-modules", &path])
        .spawn()
        .expect("Run runtime.js failed");
    Test262Suite::<Test262RuntimeCase>::new().run_async(args);
    let _ = runtime_process.wait();
    let _ = runtime_process.kill();
}

/// Default task runner (runs multiple tasks)
pub fn run_default(args: &AppArgs) {
    run_parser(args);
    run_semantic(args);
    run_codegen(args);
    // run_formatter(args);
    run_transformer(args);
    run_transpiler(args);
    run_minifier(args);
    run_estree(args);
}
