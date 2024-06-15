//! <https://github.com/microsoft/TypeScript/blob/main/src/testRunner/transpileRunner.ts>

use std::{
    fs,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;
use oxc_transformer_dts::{TransformerDts, TransformerDtsReturn};

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
    test_result: TestResult,
}

impl Case for TypeScriptTranspileCase {
    fn new(path: PathBuf, code: String) -> Self {
        let TestCaseContent { tests, settings, .. } =
            TestCaseContent::make_units_from_test(&path, &code);
        Self { path, code, units: tests, settings, test_result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.test_result
    }

    fn skip_test_case(&self) -> bool {
        !self.settings.declaration
    }

    fn run(&mut self) {
        // if !self.settings.emit_declaration_only {
        // self.run_kind(TranspileKind::Module);
        // }
        if self.settings.declaration {
            self.test_result = self.compare(TranspileKind::Declaration);
        }
    }
}

impl TypeScriptTranspileCase {
    fn compare(&self, kind: TranspileKind) -> TestResult {
        let baseline_text = self.run_kind(kind);

        // get expected text by reading its .d.ts file
        let filename = change_extension(self.path.to_str().unwrap());
        let path =
            project_root().join(TESTS_ROOT).join("baselines/reference/transpile").join(filename);

        // remove the error diagnostics lines
        let expected_text = {
            let raw_expected_text = fs::read_to_string(path).unwrap();
            let mut expected_text = String::new();
            let mut ignore = false;
            for line in raw_expected_text.split("\r\n") {
                if let Some(remain) = line.strip_prefix("//// ") {
                    ignore = remain.starts_with("[Diagnostics reported]");
                    if ignore {
                        continue;
                    }
                }
                if !ignore {
                    expected_text.push_str(line);
                    expected_text.push_str("\r\n");
                }
            }
            expected_text
        };

        // compare lines
        let baseline_lines = baseline_text.lines().filter(|s| !s.is_empty()).collect::<Vec<_>>();
        let expected_lines = expected_text.lines().filter(|s| !s.is_empty()).collect::<Vec<_>>();
        if baseline_lines.len() != expected_lines.len() {
            return TestResult::Mismatch(baseline_text, expected_text);
        }
        // compare the lines with all whitespace removed
        for (a, b) in baseline_lines.into_iter().zip(expected_lines) {
            let mut a = a.to_string();
            a.retain(|c| !c.is_whitespace());
            let mut b = b.to_string();
            b.retain(|c| !c.is_whitespace());
            if a != b {
                return TestResult::Mismatch(baseline_text, expected_text);
            }
        }
        TestResult::Passed
    }

    fn run_kind(&self, _kind: TranspileKind) -> String {
        let mut baseline_text = String::new();

        for unit in &self.units {
            baseline_text.push_str(&format!("//// [{}] ////\r\n", unit.name));
            baseline_text.push_str(&unit.content);
            if !unit.content.ends_with('\n') {
                baseline_text.push_str("\r\n");
            }
        }

        for unit in &self.units {
            let ret = transpile(&self.path, &unit.content);
            baseline_text.push_str(&format!("//// [{}] ////\r\n", change_extension(&unit.name)));
            baseline_text.push_str(&ret.source_text);
            if !ret.source_text.ends_with('\n') {
                baseline_text.push_str("\r\n");
            }
            // ignore the diagnostics for now
            // if !ret.errors.is_empty() {
            // baseline_text.push_str("\r\n\r\n//// [Diagnostics reported]\r\n");
            // for error in &ret.errors {
            // baseline_text.push_str(&error.message.to_string());
            // }
            // if !baseline_text.ends_with('\n') {
            // baseline_text.push_str("\r\n");
            // }
            // }
        }

        baseline_text
    }
}

fn change_extension(name: &str) -> String {
    Path::new(name).with_extension("").with_extension("d.ts").to_str().unwrap().to_string()
}

fn transpile(path: &Path, source_text: &str) -> TransformerDtsReturn {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    TransformerDts::new(&allocator, path, source_text, ret.trivias).build(&ret.program)
}
