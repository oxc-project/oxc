#![expect(clippy::print_stdout, clippy::disallowed_methods)]

// Core
mod runtime;
mod suite;
// Suites
mod babel;
mod misc;
mod node_compat_table;
mod test262;
mod typescript;

mod driver;
mod tools;

use std::{path::PathBuf, process::Command};

use oxc_allocator::AllocatorPool;
use oxc_tasks_common::project_root;
use runtime::Test262RuntimeCase;
use tools::estree::{AcornJsxSuite, EstreeJsxCase, EstreeTypescriptCase};

use crate::{
    babel::{BabelCase, BabelSuite},
    driver::Driver,
    misc::{MiscCase, MiscSuite},
    node_compat_table::NodeCompatSuite,
    suite::Suite,
    test262::{Test262Case, Test262Suite},
    tools::{
        codegen::{CodegenBabelCase, CodegenMiscCase, CodegenTest262Case, CodegenTypeScriptCase},
        estree::EstreeTest262Case,
        formatter::{
            FormatterBabelCase, FormatterMiscCase, FormatterTest262Case, FormatterTypeScriptCase,
        },
        minifier::{MinifierBabelCase, MinifierNodeCompatCase, MinifierTest262Case},
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

    pub fn run_default(&self, allocator_pool: &AllocatorPool) {
        self.run_parser(allocator_pool);
        self.run_semantic(allocator_pool);
        self.run_codegen(allocator_pool);
        self.run_formatter(allocator_pool);
        self.run_transformer(allocator_pool);
        self.run_transpiler(allocator_pool);
        self.run_minifier(allocator_pool);
        self.run_estree(allocator_pool);
    }

    pub fn run_parser(&self, allocator_pool: &AllocatorPool) {
        Test262Suite::<Test262Case>::new().run("parser_test262", self, allocator_pool);
        BabelSuite::<BabelCase>::new().run("parser_babel", self, allocator_pool);
        TypeScriptSuite::<TypeScriptCase>::new().run("parser_typescript", self, allocator_pool);
        MiscSuite::<MiscCase>::new().run("parser_misc", self, allocator_pool);
    }

    pub fn run_semantic(&self, allocator_pool: &AllocatorPool) {
        Test262Suite::<SemanticTest262Case>::new().run("semantic_test262", self, allocator_pool);
        BabelSuite::<SemanticBabelCase>::new().run("semantic_babel", self, allocator_pool);
        TypeScriptSuite::<SemanticTypeScriptCase>::new().run("semantic_typescript", self, allocator_pool);
        MiscSuite::<SemanticMiscCase>::new().run("semantic_misc", self, allocator_pool);
    }

    pub fn run_codegen(&self, allocator_pool: &AllocatorPool) {
        Test262Suite::<CodegenTest262Case>::new().run("codegen_test262", self, allocator_pool);
        BabelSuite::<CodegenBabelCase>::new().run("codegen_babel", self, allocator_pool);
        TypeScriptSuite::<CodegenTypeScriptCase>::new().run("codegen_typescript", self, allocator_pool);
        MiscSuite::<CodegenMiscCase>::new().run("codegen_misc", self, allocator_pool);
    }

    pub fn run_formatter(&self, allocator_pool: &AllocatorPool) {
        Test262Suite::<FormatterTest262Case>::new().run("formatter_test262", self, allocator_pool);
        BabelSuite::<FormatterBabelCase>::new().run("formatter_babel", self, allocator_pool);
        TypeScriptSuite::<FormatterTypeScriptCase>::new().run("formatter_typescript", self, allocator_pool);
        MiscSuite::<FormatterMiscCase>::new().run("formatter_misc", self, allocator_pool);
    }

    pub fn run_transformer(&self, allocator_pool: &AllocatorPool) {
        Test262Suite::<TransformerTest262Case>::new().run("transformer_test262", self, allocator_pool);
        BabelSuite::<TransformerBabelCase>::new().run("transformer_babel", self, allocator_pool);
        TypeScriptSuite::<TransformerTypeScriptCase>::new().run("transformer_typescript", self, allocator_pool);
        MiscSuite::<TransformerMiscCase>::new().run("transformer_misc", self, allocator_pool);
    }

    pub fn run_transpiler(&self, allocator_pool: &AllocatorPool) {
        TranspileRunner::<TypeScriptTranspileCase>::new().run("transpile", self, allocator_pool);
    }

    pub fn run_estree(&self, allocator_pool: &AllocatorPool) {
        Test262Suite::<EstreeTest262Case>::new().run("estree_test262", self, allocator_pool);
        AcornJsxSuite::<EstreeJsxCase>::new().run("estree_acorn_jsx", self, allocator_pool);
        TypeScriptSuite::<EstreeTypescriptCase>::new().run("estree_typescript", self, allocator_pool);
    }

    /// # Panics
    pub fn run_runtime(&self, allocator_pool: &AllocatorPool) {
        let path = workspace_root().join("src/runtime/runtime.js").to_string_lossy().to_string();
        let mut runtime_process = Command::new("node")
            .args(["--experimental-vm-modules", &path])
            .spawn()
            .expect("Run runtime.js failed");
        Test262Suite::<Test262RuntimeCase>::new().run_async(self, allocator_pool);
        let _ = runtime_process.wait();
        let _ = runtime_process.kill();
    }

    pub fn run_minifier(&self, allocator_pool: &AllocatorPool) {
        Test262Suite::<MinifierTest262Case>::new().run("minifier_test262", self, allocator_pool);
        BabelSuite::<MinifierBabelCase>::new().run("minifier_babel", self, allocator_pool);
        NodeCompatSuite::<MinifierNodeCompatCase>::new().run("minifier_node_compat", self, allocator_pool);
    }
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    let allocator_pool = AllocatorPool::new(1);
    AppArgs::default().run_default(&allocator_pool)
}
