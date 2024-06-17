//! <https://github.com/microsoft/TypeScript/blob/main/src/testRunner/transpileRunner.ts>

use std::path::{Path, PathBuf};

use oxc_allocator::Allocator;
use oxc_ast::Trivias;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_diagnostics::OxcDiagnostic;
use oxc_isolated_declarations::IsolatedDeclarations;
use oxc_parser::Parser;
use oxc_span::SourceType;

use crate::{
    project_root,
    suite::{Case, Suite, TestResult},
};

use super::meta::{Baseline, BaselineFile, CompilerSettings, TestCaseContent, TestUnitData};
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
        // get expected text by reading its .d.ts file
        let filename = change_extension(self.path.to_str().unwrap());
        let path =
            project_root().join(TESTS_ROOT).join("baselines/reference/transpile").join(filename);
        let expected = BaselineFile::parse(&path);

        let baseline = self.run_kind(kind);

        let expected_text = expected.print();
        let baseline_text = baseline.print();

        if expected.files.len() != baseline.files.len() {
            return TestResult::Mismatch(baseline_text, expected_text);
        }

        for (base, expected) in baseline.files.into_iter().zip(expected.files) {
            if base.oxc_printed != expected.oxc_printed {
                return TestResult::Mismatch(base.oxc_printed, expected.oxc_printed);
            }
        }
        TestResult::Passed
    }

    fn run_kind(&self, _kind: TranspileKind) -> BaselineFile {
        let mut files = vec![];

        for unit in &self.units {
            let mut baseline = Baseline {
                name: unit.name.clone(),
                original: unit.content.clone(),
                ..Baseline::default()
            };
            baseline.print_oxc();
            files.push(baseline);
        }

        for unit in &self.units {
            let (source_text, errors) = transpile(&self.path, &unit.content);
            let baseline = Baseline {
                name: change_extension(&unit.name),
                original: unit.content.clone(),
                oxc_printed: source_text,
                diagnostic: errors
                    .into_iter()
                    .map(|e| e.message.clone())
                    .collect::<Vec<_>>()
                    .join("\n"),
            };
            files.push(baseline);
        }

        BaselineFile { files }
    }
}

fn change_extension(name: &str) -> String {
    Path::new(name).with_extension("").with_extension("d.ts").to_str().unwrap().to_string()
}

fn transpile(path: &Path, source_text: &str) -> (String, Vec<OxcDiagnostic>) {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let ret = Parser::new(&allocator, source_text, source_type).parse();
    let ret = IsolatedDeclarations::new(&allocator).build(&ret.program);
    let printed = Codegen::<false>::new("", "", Trivias::default(), CodegenOptions::default())
        .build(&ret.program)
        .source_text;
    (printed, ret.errors)
}
