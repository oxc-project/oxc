use crate::{
    suite::{Case, Suite, TestResult},
    workspace_root,
};
use oxc::span::SourceType;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

const FIXTURES_PATH: &str = "node-compat-table";

pub struct NodeCompatSuite<T: Case> {
    test_root: PathBuf,
    cases: Vec<T>,
}

impl<T: Case> NodeCompatSuite<T> {
    pub fn new() -> Self {
        Self { test_root: PathBuf::from(FIXTURES_PATH), cases: Vec::new() }
    }
}

impl<T: Case> Suite<T> for NodeCompatSuite<T> {
    fn get_test_root(&self) -> &Path {
        &self.test_root
    }

    fn skip_test_crawl(&self) -> bool {
        // Ignore all paths since we load from testers.json
        true
    }

    fn save_test_cases(&mut self, _cases: Vec<T>) {}

    fn save_extra_test_cases(&mut self) {
        #[derive(Debug, Deserialize)]
        pub struct TestersData {
            #[serde(flatten)]
            pub versions: FxHashMap<String, FxHashMap<String, TestCase>>,
        }

        #[derive(Debug, Deserialize)]
        pub struct TestCase {
            #[expect(dead_code)]
            pub spec: Option<String>,
            pub code: String,
        }

        let testers_path = workspace_root().join(&self.test_root).join("testers.json");
        let content = fs::read_to_string(&testers_path).unwrap();
        let data = serde_json::from_str::<TestersData>(&content).unwrap();
        let mut cases = Vec::new();
        for (version, test_groups) in data.versions {
            for (test_name, test_case) in test_groups {
                let path = PathBuf::from(format!("{version}/{test_name}"));
                let case = T::new(path, test_case.code);
                if !case.skip_test_case() {
                    cases.push(case);
                }
            }
        }
        self.cases = cases;
    }

    fn get_test_cases(&self) -> &Vec<T> {
        &self.cases
    }

    fn get_test_cases_mut(&mut self) -> &mut Vec<T> {
        &mut self.cases
    }
}

pub struct NodeCompatCase {
    path: PathBuf,
    code: String,
    result: TestResult,
}

impl NodeCompatCase {
    pub fn set_result(&mut self, result: TestResult) {
        self.result = result;
    }

    pub fn source_type() -> SourceType {
        SourceType::cjs()
    }
}

impl Case for NodeCompatCase {
    fn new(path: PathBuf, code: String) -> Self {
        let wrapped_code = format!(
            // https://github.com/compat-table/node-compat-table/blob/c6ca25d77e054aaa2e227aaef00251a5272c9f0c/test.js#L22
            r"
global.__createIterableObject = function (arr, methods) {{
  methods = methods || {{}}
  if (typeof Symbol !== 'function' || !Symbol.iterator) {{
    return {{}}
  }}

  arr.length++
  var iterator = {{
    next: function () {{
      return {{
        value: arr.shift(),
        done: arr.length <= 0
      }}
    }},
    'return': methods['return'],
    'throw': methods['throw']
  }}
  var iterable = {{}}
  iterable[Symbol.iterator] = function () {{ return iterator }}

  return iterable
}}

try {{
    const result = (function() {{
        {code}
    }})();
    console.log(JSON.stringify({{ success: true, result: result }}));
}} catch (error) {{
    console.log(JSON.stringify({{ success: false, error: error.message }}));
}}
"
        );
        Self { path, code: wrapped_code, result: TestResult::ToBeRun }
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

    fn skip_test_case(&self) -> bool {
        self.path.starts_with("ESNEXT/")
    }

    fn run(&mut self) {
        let source_type = Self::source_type();
        self.result = self.execute(source_type);
    }
}
