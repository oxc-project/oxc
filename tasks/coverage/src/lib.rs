mod babel;
mod codegen;
mod minifier;
mod misc;
mod prettier;
mod runtime;
mod sourcemap;
mod suite;
mod test262;
mod typescript;

use std::{fs, path::PathBuf, process::Command, time::Duration};

use oxc_tasks_common::agent;
use runtime::{CodegenRuntimeTest262Case, V8_TEST_262_FAILED_TESTS_PATH};
use similar::DiffableStr;
use sourcemap::{SourcemapCase, SourcemapSuite};

use crate::{
    babel::{BabelCase, BabelSuite},
    codegen::{CodegenBabelCase, CodegenMiscCase, CodegenTest262Case, CodegenTypeScriptCase},
    minifier::{MinifierBabelCase, MinifierTest262Case},
    misc::{MiscCase, MiscSuite},
    prettier::{PrettierBabelCase, PrettierMiscCase, PrettierTest262Case, PrettierTypeScriptCase},
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
        self.run_prettier();
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
        SourcemapSuite::<SourcemapCase>::new().run("codegen_sourcemap", self);
    }

    pub fn run_prettier(&self) {
        Test262Suite::<PrettierTest262Case>::new().run("prettier_test262", self);
        BabelSuite::<PrettierBabelCase>::new().run("prettier_babel", self);
        TypeScriptSuite::<PrettierTypeScriptCase>::new().run("prettier_typescript", self);
        MiscSuite::<PrettierMiscCase>::new().run("prettier_misc", self);
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

    // Generate v8 test262 status file, which is used to skip failed tests
    // see https://chromium.googlesource.com/v8/v8/+/refs/heads/main/test/test262/test262.status
    #[allow(clippy::missing_panics_doc)]
    pub fn run_sync_v8_test262_status(&self) {
        let res = agent()
            .get("http://raw.githubusercontent.com/v8/v8/main/test/test262/test262.status")
            .timeout(Duration::from_secs(10))
            .call()
            .expect("Get v8 test262 status failed")
            .into_string()
            .expect("Get v8 test262 status failed");

        let mut tests = vec![];
        regex::Regex::new(r"'(.+)': \[(FAIL|SKIP)\]").unwrap().captures_iter(&res).for_each(
            |caps| {
                if let Some(name) = caps.get(1).map(|f| f.as_str()) {
                    if !name.eq("*") {
                        tests.push(name);
                    }
                }
            },
        );
        tests.sort_unstable();

        fs::write(project_root().join(V8_TEST_262_FAILED_TESTS_PATH), tests.join("\n"))
            .expect("Write v8 test262 status failed");
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
