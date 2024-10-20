#![allow(clippy::print_stdout, clippy::print_stderr, clippy::disallowed_methods)]
// Core
mod runtime;
mod suite;
// Suites
mod babel;
mod misc;
mod test262;
mod typescript;

mod driver;
mod tools;

use std::{path::PathBuf, process::Command};

use oxc_tasks_common::project_root;
use runtime::CodegenRuntimeTest262Case;
use similar::DiffableStr;

use crate::{
    babel::{BabelCase, BabelSuite},
    driver::Driver,
    misc::{MiscCase, MiscSuite},
    suite::Suite,
    test262::{Test262Case, Test262Suite},
    tools::{
        codegen::{CodegenBabelCase, CodegenMiscCase, CodegenTest262Case, CodegenTypeScriptCase},
        minifier::{MinifierBabelCase, MinifierTest262Case},
        prettier::{
            PrettierBabelCase, PrettierMiscCase, PrettierTest262Case, PrettierTypeScriptCase,
        },
        semantic::{
            SemanticBabelCase, SemanticMiscCase, SemanticTest262Case, SemanticTypeScriptCase,
        },
        transformer::{
            TransformerBabelCase, TransformerMiscCase, TransformerTest262Case,
            TransformerTypeScriptCase,
        },
    },
    typescript::{TranspileRunner, TypeScriptCase, TypeScriptSuite, TypeScriptTranspileCase},
};

pub fn workspace_root() -> PathBuf {
    project_root().join("tasks").join("coverage")
}

fn snap_root() -> PathBuf {
    workspace_root().join("snapshots")
}

#[derive(Debug, Default)]
pub struct AppArgs {
    pub debug: bool,
    pub filter: Option<String>,
    pub detail: bool,
    /// Print mismatch diff
    pub diff: bool,
}

impl AppArgs {
    fn should_print_detail(&self) -> bool {
        self.filter.is_some() || self.detail
    }

    pub fn run_all(&self) {
        self.run_parser();
        self.run_semantic();
        self.run_codegen();
        // self.run_prettier();
        self.run_transformer();
        self.run_transpiler();
        // self.run_codegen_runtime();
        self.run_minifier();
    }

    pub fn run_parser(&self) {
        Test262Suite::<Test262Case>::new().run("parser_test262", self);
        BabelSuite::<BabelCase>::new().run("parser_babel", self);
        TypeScriptSuite::<TypeScriptCase>::new().run("parser_typescript", self);
        MiscSuite::<MiscCase>::new().run("parser_misc", self);
    }

    pub fn run_semantic(&self) {
        Test262Suite::<SemanticTest262Case>::new().run("semantic_test262", self);
        BabelSuite::<SemanticBabelCase>::new().run("semantic_babel", self);
        TypeScriptSuite::<SemanticTypeScriptCase>::new().run("semantic_typescript", self);
        MiscSuite::<SemanticMiscCase>::new().run("semantic_misc", self);
    }

    pub fn run_codegen(&self) {
        Test262Suite::<CodegenTest262Case>::new().run("codegen_test262", self);
        BabelSuite::<CodegenBabelCase>::new().run("codegen_babel", self);
        TypeScriptSuite::<CodegenTypeScriptCase>::new().run("codegen_typescript", self);
        MiscSuite::<CodegenMiscCase>::new().run("codegen_misc", self);
    }

    pub fn run_prettier(&self) {
        Test262Suite::<PrettierTest262Case>::new().run("prettier_test262", self);
        BabelSuite::<PrettierBabelCase>::new().run("prettier_babel", self);
        TypeScriptSuite::<PrettierTypeScriptCase>::new().run("prettier_typescript", self);
        MiscSuite::<PrettierMiscCase>::new().run("prettier_misc", self);
    }

    pub fn run_transformer(&self) {
        Test262Suite::<TransformerTest262Case>::new().run("transformer_test262", self);
        BabelSuite::<TransformerBabelCase>::new().run("transformer_babel", self);
        TypeScriptSuite::<TransformerTypeScriptCase>::new().run("transformer_typescript", self);
        MiscSuite::<TransformerMiscCase>::new().run("transformer_misc", self);
    }

    pub fn run_transpiler(&self) {
        TranspileRunner::<TypeScriptTranspileCase>::new().run("transpile", self);
    }

    /// # Panics
    pub fn run_codegen_runtime(&self) {
        // Run runtime.js to test codegen runtime
        let mut runtime_process = Command::new("node")
            .args([
                "--experimental-vm-modules",
                workspace_root()
                    .join("src/runtime/runtime.js")
                    .to_string_lossy()
                    .as_str()
                    .unwrap_or_default(),
            ])
            .spawn()
            .expect("Run runtime.js failed");
        Test262Suite::<CodegenRuntimeTest262Case>::new().run_async("codegen_runtime_test262", self);
        let _ = runtime_process.kill();
    }

    pub fn run_minifier(&self) {
        Test262Suite::<MinifierTest262Case>::new().run("minifier_test262", self);
        BabelSuite::<MinifierBabelCase>::new().run("minifier_babel", self);
    }
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    let args = AppArgs { debug: false, filter: None, detail: false, diff: false };
    args.run_all()
}
