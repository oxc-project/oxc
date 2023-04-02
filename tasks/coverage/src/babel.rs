use std::path::{Path, PathBuf};

use oxc_ast::SourceType;
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::project_root;
use crate::suite::{Case, Suite, TestResult};

const FIXTURES_PATH: &str = "tasks/coverage/babel/packages/babel-parser/test/fixtures";

/// output.json
#[derive(Debug, Default, Clone, Deserialize)]
pub struct BabelOutput {
    pub errors: Option<Vec<String>>,
}

/// options.json
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BabelOptions {
    pub source_type: Option<String>,
    pub throws: Option<String>,
    #[serde(default)]
    pub plugins: Vec<Value>, // Can be a string or an array
    #[serde(default)]
    pub allow_return_outside_function: bool,
    #[serde(default)]
    pub allow_await_outside_function: bool,
    #[serde(default)]
    pub allow_undeclared_exports: bool,
}

pub struct BabelSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> Default for BabelSuite<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Case> BabelSuite<T> {
    #[must_use]
    pub fn new() -> Self {
        Self { test_root: project_root().join(FIXTURES_PATH), test_cases: vec![] }
    }
}

impl<T: Case> Suite<T> for BabelSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_path(&self, path: &Path) -> bool {
        let not_supported_directory = [
            "experimental",
            "es2022",
            "record-and-tuple",
            "es-record",
            "es-tuple",
            "with-pipeline",
            "v8intrinsic",
            "async-do-expression",
            "export-ns-from",
        ]
        .iter()
        .any(|p| path.to_string_lossy().contains(p));
        let incorrect_extension = path.extension().map_or(true, |ext| ext == "json" || ext == "md");
        not_supported_directory || incorrect_extension
    }

    fn save_test_cases(&mut self, tests: Vec<T>) {
        self.test_cases = tests;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.test_cases
    }
}

pub struct BabelCase {
    path: PathBuf,
    code: String,
    options: Option<BabelOptions>,
    should_fail: bool,
    result: TestResult,
}

impl BabelCase {
    fn read_file<T>(path: &Path, file_name: &'static str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let file = path.with_file_name(file_name);
        if file.exists() {
            let file = std::fs::File::open(file).unwrap();
            let reader = std::io::BufReader::new(file);
            let json: serde_json::Result<T> = serde_json::from_reader(reader);
            return json.ok();
        }
        None
    }

    fn read_output_json(path: &Path) -> Option<BabelOutput> {
        let dir = project_root().join(FIXTURES_PATH).join(path);
        if let Some(json) = Self::read_file::<BabelOutput>(&dir, "output.json") {
            return Some(json);
        }
        Self::read_file::<BabelOutput>(&dir, "output.extended.json")
    }

    /// read options.json, it exists in ancestor folders as well and they need to be merged
    fn read_options_json(path: &Path) -> Option<BabelOptions> {
        let dir = project_root().join(FIXTURES_PATH).join(path);
        let mut options_json: Option<BabelOptions> = None;
        for path in dir.ancestors().take(3) {
            if let Some(new_json) = Self::read_file::<BabelOptions>(path, "options.json") {
                if let Some(existing_json) = options_json.as_mut() {
                    if let Some(source_type) = new_json.source_type {
                        existing_json.source_type = Some(source_type);
                    }
                    if let Some(throws) = new_json.throws {
                        existing_json.throws = Some(throws);
                    }
                    existing_json.plugins.extend(new_json.plugins);
                } else {
                    options_json = Some(new_json);
                }
            }
        }
        options_json
    }

    // it is an error if:
    //   * its output.json contains an errors field
    //   * the directory contains a options.json with a "throws" field
    fn determine_should_fail(path: &Path, options: &Option<BabelOptions>) -> bool {
        let output_json = Self::read_output_json(path);

        if let Some(output_json) = output_json {
            return output_json.errors.map_or(false, |errors| !errors.is_empty());
        }

        if let Some(options) = options {
            if options.throws.is_some() {
                return true;
            }
        }

        // both files doesn't exist
        true
    }

    fn is_jsx(&self) -> bool {
        self.options.as_ref().is_some_and(|option| {
            option.plugins.iter().any(|v| v.as_str().is_some_and(|v| v == "jsx"))
        })
    }

    fn is_typescript(&self) -> bool {
        self.options.as_ref().is_some_and(|option| {
            option.plugins.iter().any(|v| {
                let string_value = v.as_str().is_some_and(|v| v == "typescript");
                let array_value =
                    v.get(0).and_then(Value::as_str).is_some_and(|s| s == "typescript");
                string_value || array_value
            })
        })
    }

    fn is_typescript_definition(&self) -> bool {
        self.options.as_ref().is_some_and(|option| {
            option.plugins.iter().filter_map(Value::as_array).any(|p| {
                let typescript =
                    p.get(0).and_then(Value::as_str).is_some_and(|s| s == "typescript");
                let dts = p
                    .get(1)
                    .and_then(Value::as_object)
                    .and_then(|v| v.get("dts"))
                    .and_then(Value::as_bool)
                    .is_some_and(|v| v);
                typescript && dts
            })
        })
    }

    fn is_module(&self) -> bool {
        self.options.as_ref().map_or(false, |option| {
            option
                .source_type
                .as_ref()
                .map_or(false, |s| matches!(s.as_str(), "module" | "unambiguous"))
        })
    }
}

impl Case for BabelCase {
    fn new(path: PathBuf, code: String) -> Self {
        let options = Self::read_options_json(&path);
        let should_fail = Self::determine_should_fail(&path, &options);
        Self { path, code, options, should_fail, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn allow_return_outside_function(&self) -> bool {
        self.options.as_ref().map_or(false, |option| option.allow_return_outside_function)
    }

    fn test_result(&self) -> &TestResult {
        &self.result
    }

    fn should_fail(&self) -> bool {
        self.should_fail
    }

    fn skip_test_case(&self) -> bool {
        let not_supported_plugins =
            ["async-do-expression", "flow", "placeholders", "decorators-legacy", "recordAndTuple"];
        self.options.as_ref().map_or(false, |option| {
            let has_not_supported_plugins = option
                .plugins
                .iter()
                .filter_map(Value::as_str)
                .any(|p| not_supported_plugins.contains(&p));
            has_not_supported_plugins
                || option.allow_await_outside_function
                || option.allow_undeclared_exports
        })
    }

    fn run(&mut self) {
        let mut source_type = SourceType::from_path(self.path()).unwrap();
        let source_type = *source_type
            .with_script(true)
            .with_jsx(self.is_jsx())
            .with_typescript(self.is_typescript())
            .with_typescript_definition(self.is_typescript_definition())
            .with_module(self.is_module());
        self.result = self.execute(source_type);
    }
}
