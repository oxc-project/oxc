mod test262_status;

use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use oxc::{
    allocator::Allocator,
    codegen::{CodeGenerator, CodegenOptions},
    minifier::{Minifier, MinifierOptions},
    parser::Parser,
    semantic::SemanticBuilder,
    span::SourceType,
    transformer::{HelperLoaderMode, TransformOptions, Transformer},
};
use oxc_tasks_common::agent;
use serde_json::json;

use crate::{
    suite::{Case, TestResult},
    test262::{Test262Case, TestFlag},
    workspace_root,
};

use test262_status::get_v8_test262_failure_paths;

static SKIP_FEATURES: &[&str] = &[
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
    "Intl.DurationFormat",
    // stage 3
    "decorators",
    "explicit-resource-management",
];

static SKIP_INCLUDES: &[&str] = &[
    // We don't preserve "toString()" on functions
    "nativeFunctionMatcher.js",
];

static SKIP_TEST_CASES: &[&str] = &[
    // node.js runtime error
    "language/eval-code",
    "language/expressions/dynamic-import",
    "language/global-code/decl-func.js",
    "language/module-code",
    // formerly S11.13.2_A5.10_T5
    "language/expressions/compound-assignment/compound-assignment-operator-calls-putvalue-lref--v",
    "language/expressions/postfix-increment/operator-x-postfix-increment-calls-putvalue-lhs-newvalue",
    "language/expressions/postfix-decrement/operator-x-postfix-decrement-calls-putvalue-lhs-newvalue",
    "language/expressions/prefix-increment/operator-prefix-increment-x-calls-putvalue-lhs-newvalue",
    "language/expressions/prefix-decrement/operator-prefix-decrement-x-calls-putvalue-lhs-newvalue",
];

pub struct Test262RuntimeCase {
    base: Test262Case,
    test_root: PathBuf,
}

impl Case for Test262RuntimeCase {
    fn new(path: PathBuf, code: String) -> Self {
        Self { base: Test262Case::new(path, code), test_root: workspace_root() }
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
        let base_path = self.path().to_string_lossy();
        let test262_path = base_path.trim_start_matches("test262/test/");
        let includes = &self.base.meta().includes;
        let features = &self.base.meta().features;
        self.base.should_fail()
            || self.base.skip_test_case()
            || base_path.contains("built-ins")
            || base_path.contains("staging")
            || base_path.contains("intl402")
            || includes.iter().any(|include| SKIP_INCLUDES.contains(&include.as_ref()))
            || features.iter().any(|feature| SKIP_FEATURES.contains(&feature.as_ref()))
            || SKIP_TEST_CASES.iter().any(|path| test262_path.starts_with(path))
            || get_v8_test262_failure_paths().iter().any(|path| {
                if let Some(path) = path.strip_suffix('*') {
                    test262_path.starts_with(path)
                } else {
                    test262_path.trim_end_matches(".js") == path
                }
            })
            || self.base.code().contains("$262")
            || self.base.code().contains("$DONOTEVALUATE()")
    }

    fn run(&mut self) {}

    async fn run_async(&mut self) {
        let code = self.get_code(false, false);
        let result = self.run_test_code("codegen", code).await;

        if result != TestResult::Passed {
            self.base.set_result(result);
            return;
        }

        let code = self.get_code(true, false);
        let result = self.run_test_code("transform", code).await;

        if result != TestResult::Passed {
            self.base.set_result(result);
            return;
        }

        // Minifier do not conform to annexB.
        let base_path = self.path().to_string_lossy();
        let test262_path = base_path.trim_start_matches("test262/test/");
        if test262_path.starts_with("annexB") {
            self.base.set_result(TestResult::Passed);
            return;
        }

        let code = self.get_code(false, true);
        let result = self.run_test_code("minify", code).await;
        self.base.set_result(result);
    }
}

impl Test262RuntimeCase {
    fn get_code(&self, transform: bool, minify: bool) -> String {
        let source_text = self.base.code();
        let is_module = self.base.meta().flags.contains(&TestFlag::Module);
        let is_only_strict = self.base.meta().flags.contains(&TestFlag::OnlyStrict);
        let source_type = SourceType::cjs().with_module(is_module);
        let allocator = Allocator::default();
        let mut program = Parser::new(&allocator, source_text, source_type).parse().program;

        if transform {
            let (symbols, scopes) =
                SemanticBuilder::new().build(&program).semantic.into_symbol_table_and_scope_tree();
            let mut options = TransformOptions::enable_all();
            options.jsx.refresh = None;
            options.helper_loader.mode = HelperLoaderMode::External;
            options.typescript.only_remove_type_imports = true;
            Transformer::new(&allocator, self.path(), options).build_with_symbols_and_scopes(
                symbols,
                scopes,
                &mut program,
            );
        }

        let mangler = if minify {
            Minifier::new(MinifierOptions { mangle: false, ..MinifierOptions::default() })
                .build(&allocator, &mut program)
                .mangler
        } else {
            None
        };

        let mut text = CodeGenerator::new()
            .with_options(CodegenOptions { minify, ..CodegenOptions::default() })
            .with_mangler(mangler)
            .build(&program)
            .code;
        if is_only_strict {
            text = format!("\"use strict\";\n{text}");
        }
        if is_module {
            text = format!("{text}\n export {{}}");
        }
        text
    }

    async fn run_test_code(&self, case: &'static str, code: String) -> TestResult {
        let is_async = self.base.meta().flags.contains(&TestFlag::Async);
        let is_module = self.base.meta().flags.contains(&TestFlag::Module);
        let mut is_raw = self.base.meta().flags.contains(&TestFlag::Raw);
        let import_dir =
            self.test_root.join(self.base.path().parent().unwrap()).to_string_lossy().to_string();

        // Tests for --> in the first line should not have raw flag
        // https://github.com/tc39/test262/issues/4020
        if self.base.path().to_string_lossy().contains("single-line-html-close-first-line-") {
            is_raw = false;
        }

        let result = request_run_code(json!({
            "code": code,
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
                    TestResult::GenericError(case, output)
                }
            }
            Err(error) => TestResult::GenericError(case, error),
        }
    }
}

async fn request_run_code(json: impl serde::Serialize + Send + 'static) -> Result<String, String> {
    tokio::spawn(async move {
        agent()
            .post("http://localhost:32055/run")
            .timeout(Duration::from_secs(4))
            .send_json(json)
            .map_err(|err| err.to_string())
            .and_then(|res| res.into_string().map_err(|err| err.to_string()))
    })
    .await
    .map_err(|err| err.to_string())?
}
