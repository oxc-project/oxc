mod babel;
mod codegen;
mod minifier;
mod misc;
mod runtime;
mod suite;
mod test262;
mod typescript;

use std::{path::PathBuf, process::Command};

use runtime::CodegenRuntimeTest262Case;
use similar::DiffableStr;

use crate::{
    babel::{BabelCase, BabelSuite},
    codegen::{CodegenBabelCase, CodegenMiscCase, CodegenTest262Case, CodegenTypeScriptCase},
    minifier::{MinifierBabelCase, MinifierTest262Case},
    misc::{MiscCase, MiscSuite},
    suite::Suite,
    test262::{Test262Case, Test262Suite},
    typescript::{TypeScriptCase, TypeScriptSuite},
};

/// # Panics
/// Invalid Project Root
pub fn project_root() -> PathBuf {
    project_root::get_project_root().unwrap()
}

#[derive(Debug, Default)]
pub struct AppArgs {
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
        self.run_codegen();
        // self.run_codegen_runtime();
        self.run_minifier();
    }

    pub fn run_parser(&self) {
        Test262Suite::<Test262Case>::new().run("parser_test262", self);
        BabelSuite::<BabelCase>::new().run("parser_babel", self);
        TypeScriptSuite::<TypeScriptCase>::new().run("parser_typescript", self);
        MiscSuite::<MiscCase>::new().run("parser_misc", self);
    }

    pub fn run_codegen(&self) {
        Test262Suite::<CodegenTest262Case>::new().run("codegen_test262", self);
        BabelSuite::<CodegenBabelCase>::new().run("codegen_babel", self);
        TypeScriptSuite::<CodegenTypeScriptCase>::new().run("codegen_typescript", self);
        MiscSuite::<CodegenMiscCase>::new().run("codegen_misc", self);
    }

    /// # Panics
    pub fn run_codegen_runtime(&self) {
        // Run runtime.js to test codegen runtime
        let mut runtime_process = Command::new("node")
            .args([
                "--experimental-vm-modules",
                project_root()
                    .join("tasks/coverage/src/runtime/runtime.js")
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
    let args = AppArgs { filter: None, detail: false, diff: false };
    args.run_all()
}
