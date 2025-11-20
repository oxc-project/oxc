use crate::workspace_root;
use oxc::span::SourceType;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

const FIXTURES_PATH: &str = "node-compat-table";

use crate::suite::{
    ExecutionOutput, ExecutionResult, ExpectedOutput, LoadedTest, ParsedTest, TestDescriptor,
    TestFilter, TestLoader, TestMetadata, TestRunner, TestSource,
};

/// NodeCompat test filter - skips ESNEXT tests
pub struct NodeCompatFilter;

impl NodeCompatFilter {
    pub const fn new() -> Self {
        Self
    }
}

impl TestFilter for NodeCompatFilter {
    fn skip_path(&self, _path: &Path) -> bool {
        // All tests come from JSON, no filesystem paths to skip
        false
    }

    fn skip_test(&self, test: &ParsedTest) -> bool {
        let path_str = test.path.to_string_lossy();

        // Skip ESNEXT tests
        if path_str.starts_with("ESNEXT/") {
            return true;
        }

        // Skip specific problematic tests
        if path_str.contains("temporal dead zone")
            || path_str == "ES2015/built-ins›well-known symbols›Symbol.toPrimitive"
            || path_str == "ES2015/misc›Proxy, internal 'get' calls›ClassDefinitionEvaluation"
            || path_str
                == "ES2015/annex b›non-strict function semantics›hoisted block-level function declaration"
        {
            return true;
        }

        false
    }
}

/// NodeCompat test source - reads from testers.json
pub struct NodeCompatSource {
    root_path: PathBuf,
}

impl NodeCompatSource {
    pub fn new() -> Self {
        Self { root_path: PathBuf::from(FIXTURES_PATH) }
    }
}

impl TestSource for NodeCompatSource {
    fn discover(&self, _filter: Option<&str>) -> Vec<TestDescriptor> {
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

        let testers_path = workspace_root().join(&self.root_path).join("testers.json");
        let content = fs::read_to_string(&testers_path).unwrap();
        let data = serde_json::from_str::<TestersData>(&content).unwrap();

        let mut descriptors = Vec::new();
        for (version, test_groups) in data.versions {
            for (test_name, test_case) in test_groups {
                let id = format!("{version}/{test_name}");
                descriptors.push(TestDescriptor::Synthetic { id, code: test_case.code });
            }
        }
        descriptors
    }
}

/// NodeCompat test loader - wraps code in harness
pub struct NodeCompatLoader;

impl NodeCompatLoader {
    pub const fn new() -> Self {
        Self
    }

    fn wrap_code(code: &str) -> String {
        format!(
            // https://github.com/williamkapke/node-compat-table/blob/c6ca25d77e054aaa2e227aaef00251a5272c9f0c/test.js#L22
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
        )
    }
}

impl TestLoader for NodeCompatLoader {
    fn load(&self, descriptor: &TestDescriptor) -> Option<LoadedTest> {
        match descriptor {
            TestDescriptor::Synthetic { id, code } => Some(LoadedTest {
                id: id.clone(),
                code: Self::wrap_code(code),
                source_type: SourceType::cjs(),
                metadata: TestMetadata::Misc,
                should_fail: false,
                expected: ExpectedOutput::None,
            }),
            TestDescriptor::FilePath(_) => None,
        }
    }
}

/// NodeCompat runner - uses minifier tool with Node.js execution
pub struct NodeCompatRunner;

impl TestRunner for NodeCompatRunner {
    fn execute_sync(&self, test: &LoadedTest) -> Option<ExecutionResult> {
        use crate::Driver;
        use oxc::minifier::{CompressOptions, CompressOptionsKeepNames};

        // Check if this test needs to preserve function names
        let keep_names = test.id.contains("\"name\" property");

        // Step 1: Minify the code with appropriate options
        let mut driver = Driver {
            compress: Some(CompressOptions {
                keep_names: if keep_names {
                    CompressOptionsKeepNames::all_true()
                } else {
                    CompressOptionsKeepNames::all_false()
                },
                ..CompressOptions::smallest()
            }),
            codegen: true,
            remove_whitespace: true,
            ..Driver::default()
        };

        driver.run(&test.code, test.source_type);
        let errors = driver.errors();

        // If minification failed, return error
        if !errors.is_empty() || driver.panicked {
            return Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Errors(vec![
                    "Failed to minify code".to_string(),
                ]),
                panicked: driver.panicked,
            });
        }

        let minified = driver.printed.clone();

        // Step 2: Execute original code in Node.js
        let original_result = Self::execute_in_node(&test.code);

        // Step 3: Execute minified code in Node.js
        let minified_result = Self::execute_in_node(&minified);

        // Step 4: Compare results
        match (&original_result, &minified_result) {
            (Ok(original_output), Ok(minified_output)) => {
                if original_output == minified_output {
                    Some(ExecutionResult {
                        output: ExecutionOutput::None,
                        error_kind: crate::suite::ErrorKind::None,
                        panicked: false,
                    })
                } else {
                    Some(ExecutionResult {
                        output: ExecutionOutput::None,
                        error_kind: crate::suite::ErrorKind::Mismatch {
                            case: "execution_result",
                            actual: minified_output.clone(),
                            expected: original_output.clone(),
                        },
                        panicked: false,
                    })
                }
            }
            (Err(e), _) | (_, Err(e)) => Some(ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Generic {
                    case: "minified_execution",
                    error: e.clone(),
                },
                panicked: false,
            }),
        }
    }

    fn name(&self) -> &'static str {
        "minifier"
    }
}

impl NodeCompatRunner {
    /// Execute JavaScript code in Node.js and return the output
    fn execute_in_node(code: &str) -> Result<String, String> {
        use std::process::Command;

        let output = Command::new("node")
            .arg("-e")
            .arg(code)
            .output()
            .map_err(|e| format!("Failed to execute node: {e}"))?;

        // Return stdout regardless of exit status
        // This is important for async tests that may exit with non-zero after
        // printing their result
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}
