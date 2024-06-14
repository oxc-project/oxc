//! <https://github.com/microsoft/TypeScript/blob/main/src/testRunner/transpileRunner.ts>

use std::path::{Path, PathBuf};

use crate::{
    project_root,
    suite::{Case, Suite, TestResult},
};

use super::meta::{CompilerSettings, TestCaseContent, TestUnitData};
use super::TESTS_ROOT;

pub struct TranspileRunner<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TranspileRunner<T> {
    pub fn new() -> Self {
        Self {
            test_root: project_root().join(TESTS_ROOT).join("cases").join("transpile"),
            test_cases: vec![],
        }
    }
}

impl<T: Case> Suite<T> for TranspileRunner<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    // fn skip_test_path(&self, _path: &Path) -> bool {
    // false
    // }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases = tests;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

#[derive(Debug, Clone, Copy)]
enum TranspileKind {
    // Module,
    Declaration,
}

pub struct TypeScriptTranspileCase {
    path: PathBuf,
    code: String,
    units: Vec<TestUnitData>,
    settings: CompilerSettings,
}

impl Case for TypeScriptTranspileCase {
    fn new(path: PathBuf, code: String) -> Self {
        let TestCaseContent { tests, settings, .. } =
            TestCaseContent::make_units_from_test(&path, &code);
        Self { path, code, units: tests, settings }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &TestResult::ToBeRun
    }

    fn skip_test_case(&self) -> bool {
        false
    }

    fn run(&mut self) {
        // if !self.settings.emit_declaration_only {
        // self.run_kind(TranspileKind::Module);
        // }
        if self.settings.declaration {
            self.run_kind(TranspileKind::Declaration);
        }
    }
}

impl TypeScriptTranspileCase {
    fn run_kind(&self, _kind: TranspileKind) {
        let mut baseline_text = String::new();

        for unit in &self.units {
            baseline_text.push_str(&format!("//// [{}] //// \r\n", unit.name));
            baseline_text.push_str(&unit.content);
            if !unit.content.ends_with('\n') {
                baseline_text.push_str("\r\n");
            }
        }

        for _unit in &self.units {}
    }
}
