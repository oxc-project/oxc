use std::path::{Path, PathBuf};

use oxc::{span::SourceType, transformer::BabelOptions};
use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

use crate::{
    suite::{Case, Suite, TestResult},
    workspace_root,
};

const FIXTURES_PATH: &str = "babel/packages/babel-parser/test/fixtures";

/// output.json
#[derive(Debug, Default, Clone, Deserialize)]
pub struct BabelOutput {
    pub errors: Option<Vec<String>>,
}

pub struct BabelSuite<T: Case> {
    test_root: PathBuf,
    test_cases: Vec<T>,
}

impl<T: Case> BabelSuite<T> {
    pub fn new() -> Self {
        Self { test_root: PathBuf::from(FIXTURES_PATH), test_cases: vec![] }
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
            "annex-b/disabled",
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

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.test_cases
    }
}

pub struct BabelCase {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    options: BabelOptions,
    should_fail: bool,
    result: TestResult,
}

impl BabelCase {
    pub fn set_result(&mut self, result: TestResult) {
        self.result = result;
    }

    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

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
        let dir = workspace_root().join(path);
        if let Some(json) = Self::read_file::<BabelOutput>(&dir, "output.json") {
            return Some(json);
        }
        Self::read_file::<BabelOutput>(&dir, "output.extended.json")
    }

    // it is an error if:
    //   * its output.json contains an errors field
    //   * the directory contains a options.json with a "throws" field
    fn determine_should_fail(path: &Path, options: &BabelOptions) -> bool {
        let output_json = Self::read_output_json(path);

        if let Some(output_json) = output_json {
            return output_json.errors.map_or(false, |errors| !errors.is_empty());
        }

        if options.throws.is_some() {
            return true;
        }

        // both files doesn't exist
        true
    }
}

impl Case for BabelCase {
    /// # Panics
    fn new(path: PathBuf, code: String) -> Self {
        let dir = workspace_root().join(&path);
        let options = BabelOptions::from_test_path(dir.parent().unwrap());
        let mut source_type = SourceType::from_path(&path)
            .unwrap()
            .with_script(true)
            .with_jsx(options.is_jsx())
            .with_typescript(options.is_typescript())
            .with_typescript_definition(options.is_typescript_definition());
        if options.is_unambiguous() {
            source_type = source_type.with_unambiguous(true);
        } else if options.is_module() {
            source_type = source_type.with_module(true);
        }
        let should_fail = Self::determine_should_fail(&path, &options);
        Self { path, code, source_type, options, should_fail, result: TestResult::ToBeRun }
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn allow_return_outside_function(&self) -> bool {
        self.options.allow_return_outside_function
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
        let has_not_supported_plugins = self.options.plugins.iter().any(|p| {
            let plugin_name = match p {
                Value::String(plugin_name) => Some(plugin_name.as_str()),
                Value::Array(a) => a.first().and_then(|plugin_name| plugin_name.as_str()),
                _ => None,
            };
            let plugin_name = plugin_name.expect("Failed to parse plugins config");
            not_supported_plugins.contains(&plugin_name)
        });
        has_not_supported_plugins
            || self.options.allow_await_outside_function
            || self.options.allow_undeclared_exports
    }

    fn run(&mut self) {
        let source_type = self.source_type();
        self.result = self.execute(source_type);
    }
}
