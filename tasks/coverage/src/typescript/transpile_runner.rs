//! <https://github.com/microsoft/TypeScript/blob/main/src/testRunner/transpileRunner.ts>

use std::path::{Path, PathBuf};

use oxc::{
    allocator::Allocator,
    codegen::CodeGenerator,
    diagnostics::OxcDiagnostic,
    isolated_declarations::{IsolatedDeclarations, IsolatedDeclarationsOptions},
    parser::Parser,
    span::SourceType,
};

use super::{
    meta::{Baseline, BaselineFile},
    TypeScriptCase, TESTS_ROOT,
};
use crate::{
    suite::{Case, Suite, TestResult},
    workspace_root,
};

pub struct TranspileRunner<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TranspileRunner<T> {
    pub fn new() -> Self {
        Self {
            test_root: workspace_root().join(TESTS_ROOT).join("cases").join("transpile"),
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
    base: TypeScriptCase,
}

impl Case for TypeScriptTranspileCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: TypeScriptCase::new(path, code) }
    }

    fn code(&self) -> &str {
        self.base.code()
    }

    fn path(&self) -> &Path {
        self.base.path()
    }

    fn test_result(&self) -> &TestResult {
        self.base.test_result()
    }

    fn skip_test_case(&self) -> bool {
        !self.base.settings.declaration
    }

    fn run(&mut self) {
        // if !self.settings.emit_declaration_only {
        // self.run_kind(TranspileKind::Module);
        // }
        if self.base.settings.declaration {
            self.base.result = self.compare(TranspileKind::Declaration);
        }
    }
}

impl TypeScriptTranspileCase {
    fn compare(&self, kind: TranspileKind) -> TestResult {
        // get expected text by reading its .d.ts file
        let path = self.path().strip_prefix("typescript/tests/cases/transpile").unwrap();
        let filename = change_extension(path.to_str().unwrap());
        let path =
            workspace_root().join(TESTS_ROOT).join("baselines/reference/transpile").join(filename);
        let expected = BaselineFile::parse(&path);

        let baseline = self.run_kind(kind);

        let expected_text = expected.print();
        let baseline_text = baseline.print();

        if expected.files.len() != baseline.files.len() {
            return TestResult::Mismatch("Mismatch", baseline_text, expected_text);
        }

        for (base, expected) in baseline.files.iter().zip(expected.files) {
            if expected.original_diagnostic.is_empty() {
                if base.oxc_printed != expected.oxc_printed {
                    return TestResult::Mismatch(
                        "Mismatch",
                        base.oxc_printed.clone(),
                        expected.oxc_printed,
                    );
                }
            } else {
                let matched = base.oxc_diagnostics.iter().zip(expected.original_diagnostic).all(
                    |(base_diagnostic, expected_diagnostic)| {
                        expected_diagnostic.contains(&base_diagnostic.to_string())
                    },
                );
                if !matched {
                    let snapshot =
                        format!("\n#### {:?} ####\n{}", self.path(), baseline.snapshot());
                    return TestResult::CorrectError(snapshot, false);
                }
            }
        }

        TestResult::Passed
    }

    fn run_kind(&self, _kind: TranspileKind) -> BaselineFile {
        let mut files = vec![];

        for unit in &self.base.units {
            let mut baseline = Baseline {
                name: unit.name.clone(),
                original: unit.content.clone(),
                ..Baseline::default()
            };
            baseline.print_oxc();
            files.push(baseline);
        }

        for unit in &self.base.units {
            let (source_text, errors) = transpile(self.path(), &unit.content);
            let baseline = Baseline {
                name: change_extension(&unit.name),
                original: unit.content.clone(),
                original_diagnostic: Vec::default(),
                oxc_printed: source_text,
                oxc_diagnostics: errors,
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
    let ret =
        IsolatedDeclarations::new(&allocator, IsolatedDeclarationsOptions { strip_internal: true })
            .build(&ret.program);
    let printed = CodeGenerator::new().build(&ret.program).code;
    (printed, ret.errors)
}
