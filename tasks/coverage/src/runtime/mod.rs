use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use oxc_tasks_common::{agent, project_root};
use phf::{phf_set, Set};

use oxc_allocator::Allocator;
use oxc_codegen::{Codegen, CodegenOptions};
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::json;

use crate::{
    suite::{Case, TestResult},
    test262::{Test262Case, TestFlag},
};

static SKIP_EVALUATING_FEATURES: Set<&'static str> = phf_set! {
  // Node's version of V8 doesn't implement these
  "hashbang",
  "legacy-regexp",
  "regexp-duplicate-named-groups",
  "symbols-as-weakmap-keys",
  "tail-call-optimization",

  // We don't care about API-related things
  "ArrayBuffer",
  "change-array-by-copy",
  "DataView",
  "resizable-arraybuffer",
  "ShadowRealm",
  "cross-realm",
  "SharedArrayBuffer",
  "String.prototype.toWellFormed",
  "Symbol.match",
  "Symbol.replace",
  "Symbol.unscopables",
  "Temporal",
  "TypedArray",

  // Added in oxc
  "Array.fromAsync",
  "IsHTMLDDA",
  "iterator-helpers",
  "set-methods",
  "array-grouping",

  // stage 2
  "Intl.DurationFormat"
};

static SKIP_EVALUATING_THESE_INCLUDES: Set<&'static str> = phf_set! {
    // We don't preserve "toString()" on functions
    "nativeFunctionMatcher.js",
};

static SKIP_TEST_CASES: Set<&'static str> = phf_set! {
    // For some unknown reason these tests are unstable, so we'll skip them for now.
    "language/identifiers/start-unicode"
};

const FIXTURES_PATH: &str = "tasks/coverage/test262/test";

pub struct CodegenRuntimeTest262Case {
    base: Test262Case,
    test_root: PathBuf,
}

impl Case for CodegenRuntimeTest262Case {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: Test262Case::new(path, code), test_root: project_root().join(FIXTURES_PATH) }
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
        let base_path = self.base.path().to_string_lossy();
        self.base.should_fail()
            || base_path.starts_with("built-ins")
            || base_path.starts_with("staging")
            || base_path.starts_with("intl402")
            || self
                .base
                .meta()
                .includes
                .iter()
                .any(|include| SKIP_EVALUATING_THESE_INCLUDES.contains(include))
            || self
                .base
                .meta()
                .features
                .iter()
                .any(|feature| SKIP_EVALUATING_FEATURES.contains(feature))
            || SKIP_TEST_CASES.iter().any(|path| base_path.contains(path))
            || self.base.code().contains("$262")
            || self.base.code().contains("$DONOTEVALUATE()")
    }

    fn run(&mut self) {}

    async fn run_async(&mut self) {
        let result = async {
            let codegen_source_text = {
                let source_text = self.base.code();
                let is_module = self.base.meta().flags.contains(&TestFlag::Module);
                let is_only_strict = self.base.meta().flags.contains(&TestFlag::OnlyStrict);
                let source_type = SourceType::default().with_module(is_module);
                let allocator = Allocator::default();
                let program = Parser::new(&allocator, source_text, source_type).parse().program;
                let mut text =
                    Codegen::<false>::new(source_text.len(), CodegenOptions).build(&program);
                if is_only_strict {
                    text = format!("\"use strict\";\n{text}");
                }
                if is_module {
                    text = format!("{text}\n export {{}}");
                }
                text
            };

            self.run_test_code(codegen_source_text).await
        }
        .await;
        self.base.set_result(result);
    }
}

impl CodegenRuntimeTest262Case {
    async fn run_test_code(&self, codegen_text: String) -> TestResult {
        let is_async = self.base.meta().flags.contains(&TestFlag::Async);
        let is_module = self.base.meta().flags.contains(&TestFlag::Module);
        let is_raw = self.base.meta().flags.contains(&TestFlag::Raw);
        let import_dir = self
            .test_root
            .join(self.base.path().parent().expect("Failed to get parent directory"))
            .to_string_lossy()
            .to_string();

        let result = request_run_code(json!({
            "code": codegen_text,
            "includes": self.base.meta().includes,
            "isAsync": is_async,
            "isModule": is_module,
            "isRaw": is_raw,
            "importDir": import_dir
        }))
        .await;

        match result {
            Ok(output) => {
                if output.is_empty() {
                    TestResult::Passed
                } else {
                    if let Some(negative) = &self.base.meta().negative {
                        if negative.phase.is_runtime()
                            && output.starts_with(&negative.error_type.to_string())
                        {
                            return TestResult::Passed;
                        }
                    }
                    TestResult::RuntimeError(output)
                }
            }
            Err(error) => TestResult::RuntimeError(error),
        }
    }
}

async fn request_run_code(json: impl serde::Serialize + Send + 'static) -> Result<String, String> {
    tokio::spawn(async move {
        agent()
            .post("http://localhost:32055/run")
            .timeout(Duration::from_secs(10))
            .send_json(json)
            .map_err(|err| err.to_string())
            .and_then(|res| res.into_string().map_err(|err| err.to_string()))
    })
    .await
    .map_err(|err| err.to_string())?
}
