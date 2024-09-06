mod meta;
mod transpile_runner;

use std::path::{Path, PathBuf};

use self::meta::{CompilerSettings, TestCaseContent, TestUnitData};
pub use self::transpile_runner::{TranspileRunner, TypeScriptTranspileCase};
use crate::suite::{Case, Suite, TestResult};

const TESTS_ROOT: &str = "typescript/tests";

pub struct TypeScriptSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TypeScriptSuite<T> {
    pub fn new() -> Self {
        Self { test_root: PathBuf::from(TESTS_ROOT).join("cases"), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for TypeScriptSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        // stack overflows in compiler tests
        #[cfg(any(coverage, coverage_nightly))]
        let supported_paths = ["conformance"].iter().any(|p| path.to_string_lossy().contains(p));
        #[cfg(not(any(coverage, coverage_nightly)))]
        let supported_paths =
            ["conformance", "compiler"].iter().any(|p| path.to_string_lossy().contains(p));
        let unsupported_tests = [
            // these 2 relies on the ts "target" option
            "functionWithUseStrictAndSimpleParameterList.ts",
            "parameterInitializerBeforeDestructuringEmit.ts",
            // these also relies on "target: es5" option w/ RegExp `u` flag
            "unicodeExtendedEscapesInRegularExpressions01.ts",
            "unicodeExtendedEscapesInRegularExpressions02.ts",
            "unicodeExtendedEscapesInRegularExpressions03.ts",
            "unicodeExtendedEscapesInRegularExpressions04.ts",
            "unicodeExtendedEscapesInRegularExpressions05.ts",
            "unicodeExtendedEscapesInRegularExpressions06.ts",
            "unicodeExtendedEscapesInRegularExpressions08.ts",
            "unicodeExtendedEscapesInRegularExpressions09.ts",
            "unicodeExtendedEscapesInRegularExpressions10.ts",
            "unicodeExtendedEscapesInRegularExpressions11.ts",
            "unicodeExtendedEscapesInRegularExpressions13.ts",
            "unicodeExtendedEscapesInRegularExpressions15.ts",
            "unicodeExtendedEscapesInRegularExpressions16.ts",
            "unicodeExtendedEscapesInRegularExpressions18.ts",
        ]
        .iter()
        .any(|p| path.to_string_lossy().contains(p));
        !supported_paths || unsupported_tests
    }

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

pub struct TypeScriptCase {
    path: PathBuf,
    pub code: String,
    pub units: Vec<TestUnitData>,
    pub settings: CompilerSettings,
    error_files: Vec<String>,
    pub result: TestResult,
}

impl Case for TypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let TestCaseContent { tests, settings, error_files } =
            TestCaseContent::make_units_from_test(&path, &code);
        Self { path, code, units: tests, settings, error_files, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        !self.error_files.is_empty()
    }

    fn always_strict(&self) -> bool {
        self.settings.always_strict
    }

    fn run(&mut self) {
        let units = self.units.clone();
        for unit in units {
            self.code.clone_from(&unit.content);
            self.result = self.execute(unit.source_type);
            if self.result != TestResult::Passed {
                return;
            }
        }
        self.result = TestResult::Passed;
    }
}
