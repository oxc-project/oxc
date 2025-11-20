use std::path::Path;

use serde_json::json;

use oxc::{
    allocator::Allocator,
    codegen::{Codegen, CodegenOptions},
    minifier::{Minifier, MinifierOptions},
    parser::Parser,
    semantic::SemanticBuilder,
    span::SourceType,
    transformer::{HelperLoaderMode, TransformOptions, Transformer},
};
use oxc_tasks_common::agent;

use crate::workspace_root;

mod test262_status;
use test262_status::get_v8_test262_failure_paths;

// Helper function extracted from removed Test262RuntimeCase impl
async fn request_run_code(json: impl serde::Serialize + Send + 'static) -> Result<String, String> {
    tokio::spawn(async move {
        agent()
            .post("http://localhost:32055/run")
            .send_json(json)
            .map_err(|err| err.to_string())
            .and_then(|mut res| res.body_mut().read_to_string().map_err(|err| err.to_string()))
    })
    .await
    .map_err(|err| err.to_string())?
}

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
    "source-phase-imports",
    "import-defer",
];

static SKIP_INCLUDES: &[&str] = &[
    // We don't preserve "toString()" on functions
    "nativeFunctionMatcher.js",
];

static SKIP_TEST_CASES: &[&str] = &[
    // node.js runtime error
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

static SKIP_ESID: &[&str] = &["sec-privatefieldget", "sec-privatefieldset"];

use crate::{
    suite::{ExecutionOutput, ExecutionResult, LoadedTest, TestFilter, TestMetadata, TestRunner},
    test262::{Negative, TestFlag},
};
use futures::future::BoxFuture;

/// Runtime test runner with async execution
pub struct RuntimeRunner;

impl TestRunner for RuntimeRunner {
    fn execute_async(&self, test: &LoadedTest) -> BoxFuture<'static, Option<ExecutionResult>> {
        // Clone data needed for async closure
        let test_id = test.id.clone();
        let code = test.code.clone();
        let source_type = test.source_type;
        let metadata = test.metadata.clone();

        Box::pin(async move {
            let TestMetadata::Test262 { flags, negative, includes, .. } = &metadata else {
                return Some(ExecutionResult {
                    output: ExecutionOutput::None,
                    error_kind: crate::suite::ErrorKind::Errors(vec![
                        "Not a Test262 test".to_string(),
                    ]),
                    panicked: false,
                });
            };

            // Test Phase 1: Codegen
            let codegen_code = Self::get_code(&code, source_type, flags, false, false);
            let codegen_result = Self::run_test_code(
                &test_id,
                &codegen_code,
                flags,
                negative.as_ref(),
                includes,
                "codegen",
            )
            .await;

            if codegen_result.error_kind.has_errors() {
                return Some(codegen_result);
            }

            // Test Phase 2: Transform
            let transform_code = Self::get_code(&code, source_type, flags, true, false);
            let transform_result = Self::run_test_code(
                &test_id,
                &transform_code,
                flags,
                negative.as_ref(),
                includes,
                "transform",
            )
            .await;

            if transform_result.error_kind.has_errors() {
                return Some(transform_result);
            }

            // Test Phase 3: Minify (with early exit conditions)
            let base_path = test_id.as_str();
            let test262_path = base_path.trim_start_matches("test262/test/");

            // Skip minify for annexB tests
            if test262_path.starts_with("annexB") {
                return Some(ExecutionResult {
                    output: ExecutionOutput::None,
                    error_kind: crate::suite::ErrorKind::None,
                    panicked: false,
                });
            }

            // Skip minify for non-strict code
            if flags.contains(&TestFlag::NoStrict) {
                return Some(ExecutionResult {
                    output: ExecutionOutput::None,
                    error_kind: crate::suite::ErrorKind::None,
                    panicked: false,
                });
            }

            // Skip minify for fn-name-cover.js tests
            if test262_path.ends_with("fn-name-cover.js") {
                return Some(ExecutionResult {
                    output: ExecutionOutput::None,
                    error_kind: crate::suite::ErrorKind::None,
                    panicked: false,
                });
            }

            let minify_code = Self::get_code(&code, source_type, flags, false, true);
            let minify_result = Self::run_test_code(
                &test_id,
                &minify_code,
                flags,
                negative.as_ref(),
                includes,
                "minify",
            )
            .await;

            Some(minify_result)
        })
    }

    fn name(&self) -> &'static str {
        "runtime"
    }
}

impl RuntimeRunner {
    fn get_code(
        source_text: &str,
        source_type: SourceType,
        flags: &[TestFlag],
        transform: bool,
        minify: bool,
    ) -> String {
        let allocator = Allocator::default();
        let mut program = Parser::new(&allocator, source_text, source_type).parse().program;

        if transform {
            let scoping = SemanticBuilder::new().build(&program).semantic.into_scoping();
            let mut options = TransformOptions::enable_all();
            options.jsx.refresh = None;
            options.helper_loader.mode = HelperLoaderMode::External;
            options.typescript.only_remove_type_imports = true;
            Transformer::new(&allocator, Path::new(""), &options)
                .build_with_scoping(scoping, &mut program);
        }

        let symbol_table = if minify {
            Minifier::new(MinifierOptions { mangle: None, ..MinifierOptions::default() })
                .minify(&allocator, &mut program)
                .scoping
        } else {
            None
        };

        let mut text = Codegen::new()
            .with_options(if minify { CodegenOptions::minify() } else { CodegenOptions::default() })
            .with_scoping(symbol_table)
            .build(&program)
            .code;

        if flags.contains(&TestFlag::OnlyStrict) {
            text = format!("\"use strict\";\n{text}");
        }
        if flags.contains(&TestFlag::Module) {
            text = format!("{text}\n export {{}}");
        }
        text
    }

    async fn run_test_code(
        test_id: &str,
        code: &str,
        flags: &[TestFlag],
        negative: Option<&Negative>,
        includes: &[Box<str>],
        phase: &'static str,
    ) -> ExecutionResult {
        // Get import directory from test path
        let import_dir = if let Some(parent) = Path::new(test_id).parent() {
            workspace_root().join(parent).to_string_lossy().to_string()
        } else {
            workspace_root().to_string_lossy().to_string()
        };

        let result = request_run_code(json!({
            "code": code,
            "includes": includes,
            "isAsync": flags.contains(&TestFlag::Async),
            "isModule": flags.contains(&TestFlag::Module),
            "isRaw": flags.contains(&TestFlag::Raw),
            "importDir": import_dir
        }))
        .await;

        match result {
            Ok(output) => {
                if output.is_empty() {
                    ExecutionResult {
                        output: ExecutionOutput::None,
                        error_kind: crate::suite::ErrorKind::None,
                        panicked: false,
                    }
                } else if let Some(neg) = negative {
                    if neg.phase.is_runtime() && output.starts_with(&neg.error_type.to_string()) {
                        ExecutionResult {
                            output: ExecutionOutput::None,
                            error_kind: crate::suite::ErrorKind::None,
                            panicked: false,
                        }
                    } else {
                        ExecutionResult {
                            output: ExecutionOutput::None,
                            error_kind: crate::suite::ErrorKind::Errors(vec![format!(
                                "{}: {}",
                                phase, output
                            )]),
                            panicked: false,
                        }
                    }
                } else {
                    ExecutionResult {
                        output: ExecutionOutput::None,
                        error_kind: crate::suite::ErrorKind::Errors(vec![format!(
                            "{}: {}",
                            phase, output
                        )]),
                        panicked: false,
                    }
                }
            }
            Err(error) => ExecutionResult {
                output: ExecutionOutput::None,
                error_kind: crate::suite::ErrorKind::Errors(vec![format!("{}: {}", phase, error)]),
                panicked: false,
            },
        }
    }
}

/// Runtime Test Filter - Complex filtering logic for runtime tests
pub struct RuntimeFilter {
    base: crate::test262::Test262Filter,
}

impl RuntimeFilter {
    pub const fn new() -> Self {
        Self { base: crate::test262::Test262Filter::new() }
    }
}

impl TestFilter for RuntimeFilter {
    fn skip_path(&self, path: &Path) -> bool {
        let base_path = path.to_string_lossy();
        let test262_path = base_path.trim_start_matches("test262/test/");

        // Skip built-ins, staging, intl402
        base_path.contains("built-ins")
            || base_path.contains("staging")
            || base_path.contains("intl402")
            || SKIP_TEST_CASES.iter().any(|skip_path| test262_path.starts_with(skip_path))
            || self.base.skip_path(path)
    }

    fn skip_test(&self, test: &crate::suite::ParsedTest) -> bool {
        let TestMetadata::Test262 { esid, features, includes, .. } = &test.metadata else {
            return true;
        };

        let base_path = test.path.to_string_lossy();
        let test262_path = base_path.trim_start_matches("test262/test/");

        // Skip if should_fail
        if test.should_fail {
            return true;
        }

        // Skip based on ESID
        if let Some(esid) = esid
            && SKIP_ESID.contains(&esid.as_ref())
        {
            return true;
        }

        // Skip based on includes
        if includes.iter().any(|include| SKIP_INCLUDES.contains(&include.as_ref())) {
            return true;
        }

        // Skip based on features
        if features.iter().any(|feature| SKIP_FEATURES.contains(&feature.as_ref())) {
            return true;
        }

        // Skip V8 test262 failure paths
        if get_v8_test262_failure_paths().iter().any(|path| {
            if let Some(path) = path.strip_suffix('*') {
                test262_path.starts_with(path)
            } else {
                test262_path.trim_end_matches(".js") == path
            }
        }) {
            return true;
        }

        // Skip tests with $262 or $DONOTEVALUATE()
        if test.code.contains("$262") || test.code.contains("$DONOTEVALUATE()") {
            return true;
        }

        // Delegate to base filter
        self.base.skip_test(test)
    }
}
