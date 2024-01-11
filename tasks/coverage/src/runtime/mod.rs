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

    fn run(&mut self) {
        let result = {
            let source_text = self.base.code();
            let is_module = self.base.meta().flags.contains(&TestFlag::Module);
            let is_only_strict = self.base.meta().flags.contains(&TestFlag::OnlyStrict);
            let source_type = SourceType::default()
                .with_module(is_module)
                .with_always_strict(self.base.meta().flags.contains(&TestFlag::OnlyStrict));
            let allocator = Allocator::default();
            let program = Parser::new(&allocator, source_text, source_type).parse().program;
            let mut codegen_source_text =
                Codegen::<false>::new(source_text.len(), CodegenOptions).build(&program);
            if is_only_strict {
                codegen_source_text = format!("\"use strict\";\n{codegen_source_text}");
            }
            if is_module {
                codegen_source_text = format!("{codegen_source_text}\n export {{}}");
            }

            self.run_test_code(codegen_source_text.as_str())
        };
        self.base.set_result(result);
    }
}

impl CodegenRuntimeTest262Case {
    fn run_test_code(&self, codegen_text: &str) -> TestResult {
        let is_async = self.base.meta().flags.contains(&TestFlag::Async);
        let is_module = self.base.meta().flags.contains(&TestFlag::Module);
        let is_raw = self.base.meta().flags.contains(&TestFlag::Raw);
        let import_dir = self
            .test_root
            .join(self.base.path().parent().map_or_else(|| unreachable!(), |p| p))
            .to_string_lossy()
            .to_string();
        let result = agent()
            .post("http://localhost:32055/run")
            .timeout(Duration::from_secs(10))
            .send_json(&json!({
                "code": codegen_text,
                "includes": self.base.meta().includes,
                "isAsync": is_async,
                "isModule": is_module,
                "isRaw": is_raw,
                "importDir": import_dir
            }))
            .map(|res| res.into_string().unwrap_or_default());

        if let Err(error) = result {
            TestResult::RuntimeError(error.to_string())
        } else if let Ok(output) = result {
            if output.is_empty() {
                TestResult::Passed
            } else {
                TestResult::RuntimeError(output)
            }
        } else {
            unreachable!()
        }
    }
}
