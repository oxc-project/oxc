use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use oxc_span::SourceType;
use regex::Regex;

use crate::{
    project_root,
    suite::{Case, Suite, TestResult},
};

const TESTS_ROOT: &str = "tasks/coverage/typescript/tests/";

lazy_static::lazy_static! {
    // Returns a match for a test option. Test options have the form `// @name: value`
    static ref META_OPTIONS: Regex = Regex::new(r"(?m)^/{2}[[:space:]]*@(?P<name>[[:word:]]+)[[:space:]]*:[[:space:]]*(?P<value>[^\r\n]*)").unwrap();
    static ref TEST_BRACES: Regex = Regex::new(r"^[[:space:]]*[{|}][[:space:]]*$").unwrap();
}

pub struct TypeScriptSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> TypeScriptSuite<T> {
    pub fn new() -> Self {
        Self { test_root: project_root().join(TESTS_ROOT).join("cases"), test_cases: vec![] }
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
    code: String,
    source_type: SourceType,
    result: TestResult,
    meta: TypeScriptTestMeta,
}

impl TypeScriptCase {
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    pub fn set_result(&mut self, result: TestResult) {
        self.result = result;
    }
}

impl Case for TypeScriptCase {
    fn new(path: PathBuf, code: String) -> Self {
        let meta = TypeScriptTestMeta::from_source(&path, &code);
        let compiler_options = &meta.options;
        let is_module = ["esnext", "es2022", "es2020", "es2015"]
            .into_iter()
            .any(|module| compiler_options.modules.contains(&module.to_string()));
        let source_type = SourceType::from_path(&path)
            .unwrap()
            .with_script(true)
            .with_module(is_module)
            .with_jsx(!compiler_options.jsx.is_empty())
            .with_typescript_definition(compiler_options.declaration);
        Self { path, code, source_type, result: TestResult::ToBeRun, meta }
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
        !self.meta.error_files.is_empty()
    }

    fn skip_test_case(&self) -> bool {
        // skip multi-file test cases for now
        self.meta.tests.len() > 1
    }

    fn run(&mut self) {
        self.result = self.execute(self.source_type);
    }
}

struct TypeScriptTestMeta {
    pub tests: Vec<TestUnitData>,
    pub options: CompilerOptions,
    error_files: Vec<String>,
}

impl TypeScriptTestMeta {
    /// TypeScript supports multiple files in a single test case.
    /// These files start with `// @<option-name>: <option-value>` and are followed by the file's content.
    /// This function extracts the individual files with their content and drops unsupported files.
    /// See `makeUnitsFromTest` in `harnessIO.ts` from TypeScript.
    pub fn from_source(path: &Path, code: &str) -> Self {
        let mut current_file_options: HashMap<String, String> = HashMap::default();
        let mut current_file_name: Option<String> = None;
        let mut test_unit_data: Vec<TestUnitData> = vec![];
        let mut current_file_content = String::new();

        for line in code.lines() {
            if let Some(option) = META_OPTIONS.captures(line) {
                let meta_name = option.name("name").unwrap().as_str();
                let meta_name = meta_name.to_lowercase();
                let meta_value = option.name("value").unwrap().as_str();
                let meta_value = meta_value.trim();
                if meta_name != "filename" {
                    current_file_options.insert(meta_name.clone(), meta_value.to_string());
                    continue;
                }
                if let Some(file_name) = current_file_name.take() {
                    test_unit_data.push(TestUnitData {
                        name: file_name,
                        content: std::mem::take(&mut current_file_content),
                    });
                }
                current_file_name = Some(meta_value.to_string());
            } else {
                if !current_file_content.is_empty() {
                    current_file_content.push('\n');
                }
                current_file_content.push_str(line);
            }
        }

        // normalize the fileName for the single file case
        let file_name = if !test_unit_data.is_empty() || current_file_name.is_some() {
            current_file_name.unwrap()
        } else {
            path.file_name().unwrap().to_string_lossy().to_string()
        };

        test_unit_data.push(TestUnitData {
            name: file_name,
            content: std::mem::take(&mut current_file_content),
        });

        let options = CompilerOptions::new(&current_file_options);
        let error_files = Self::get_error_files(path, &options);
        Self { tests: test_unit_data, options, error_files }
    }

    // TypeScript error files can be:
    //   * `filename(module=es2022).errors.txt`
    //   * `filename(target=esnext).errors.txt`
    //   * `filename.errors.txt`
    fn get_error_files(path: &Path, options: &CompilerOptions) -> Vec<String> {
        let file_name = path.file_stem().unwrap().to_string_lossy();
        let root = project_root().join(TESTS_ROOT).join("baselines/reference");
        let mut suffixes = vec![String::new()];
        suffixes.extend(options.modules.iter().map(|module| format!("(module={module})")));
        suffixes.extend(options.targets.iter().map(|target| format!("(target={target})")));
        suffixes.extend(options.jsx.iter().map(|jsx| format!("(jsx={jsx})")));
        let mut error_files = vec![];
        for suffix in suffixes {
            let error_path = root.join(format!("{file_name}{suffix}.errors.txt"));
            if error_path.exists() {
                let error_file = fs::read_to_string(error_path).unwrap();
                error_files.push(error_file);
            }
        }
        error_files
    }
}

#[derive(Debug)]
#[allow(unused)]
struct TestUnitData {
    name: String,
    content: String,
}

#[derive(Debug)]
#[allow(unused)]
struct CompilerOptions {
    pub modules: Vec<String>,
    pub targets: Vec<String>,
    pub strict: bool,
    pub jsx: Vec<String>, // 'react', 'preserve'
    pub declaration: bool,
    pub always_strict: bool, // Ensure 'use strict' is always emitted.
    pub allow_unreachable_code: bool,
    pub allow_unused_labels: bool,
    pub no_fallthrough_cases_in_switch: bool,
}

impl CompilerOptions {
    pub fn new(options: &HashMap<String, String>) -> Self {
        Self {
            modules: Self::split_value_options(options.get("module")),
            targets: Self::split_value_options(options.get("target")),
            strict: Self::value_to_boolean(options.get("strict"), false),
            jsx: Self::split_value_options(options.get("jsx")),
            declaration: Self::value_to_boolean(options.get("declaration"), false),
            always_strict: Self::value_to_boolean(options.get("alwaysstrict"), false),
            allow_unreachable_code: Self::value_to_boolean(
                options.get("allowunreachablecode"),
                true,
            ),
            allow_unused_labels: Self::value_to_boolean(options.get("allowunusedlabels"), true),
            no_fallthrough_cases_in_switch: Self::value_to_boolean(
                options.get("nofallthroughcasesinswitch"),
                false,
            ),
        }
    }

    fn split_value_options(value: Option<&String>) -> Vec<String> {
        value
            .map(|value| value.split(',').map(|s| s.trim().to_lowercase()).collect())
            .unwrap_or_default()
    }

    fn value_to_boolean(value: Option<&String>, default: bool) -> bool {
        match value.map(AsRef::as_ref) {
            Some("true") => true,
            Some("false") => false,
            _ => default,
        }
    }
}
