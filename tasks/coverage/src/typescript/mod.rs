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
            // TS18010, but requires JSDoc TS parsing
            "privateNamesIncompatibleModifiersJs.ts",
            // Exporting JSDoc types from `.js`
            "importingExportingTypes.ts",
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
    error_codes: Vec<String>,
    pub result: TestResult,
}

impl TypeScriptCase {
    /// Simple check for usage such as `semantic`.
    /// `should_fail()` will return `true` only if there are still error codes remaining
    /// after filtering out the not-supported ones.
    pub fn should_fail_with_any_error_codes(&self) -> bool {
        !self.error_codes.is_empty()
    }
}

impl Case for TypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let TestCaseContent { tests, settings, error_codes } =
            TestCaseContent::make_units_from_test(&path, &code);
        Self { path, code, units: tests, settings, error_codes, result: TestResult::ToBeRun }
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
        // If there are still error codes to be supported, it should fail
        self.error_codes.iter().any(|code| !NOT_SUPPORTED_ERROR_CODES.contains(code.as_str()))
    }

    fn always_strict(&self) -> bool {
        self.settings.always_strict
    }

    fn run(&mut self) {
        let result = self
            .units
            .iter()
            .map(|unit| self.parse(&unit.content, unit.source_type))
            .find(Result::is_err)
            .unwrap_or(Ok(()));
        self.result = self.evaluate_result(result);
    }
}

// TODO: Filter out more not-supported error codes here
static NOT_SUPPORTED_ERROR_CODES: phf::Set<&'static str> = phf::phf_set![
    "2315",  // Type 'U' is not generic.
    "18028", // Private identifiers are only available when targeting ECMAScript 2015 and higher.
    "18033", // Type 'Number' is not assignable to type 'number' as required for computed enum member values.
    "18035", // Invalid value for 'jsxFragmentFactory'. '234' is not a valid identifier or qualified-name.
    "18042", // 'Prop' is a type and cannot be imported in JavaScript files. Use 'import("./component").Prop' in a JSDoc type annotation.
    "18043", // Types cannot appear in export declarations in JavaScript files.
    "18045", // Properties with the 'accessor' modifier are only available when targeting ECMAScript 2015 and higher.
    "18046", // 'x' is of type 'unknown'.
    "18047", // 'x' is possibly 'null'.
    "18048", // 'x' is possibly 'undefined'.
    "18049", // 'x' is possibly 'null' or 'undefined'.
    "18055", // 'A.a' has a string type, but must have syntactically recognizable string syntax when 'isolatedModules' is enabled.
    "18057", // String literal import and export names are not supported when the '--module' flag is set to 'es2015' or 'es2020'.
];
