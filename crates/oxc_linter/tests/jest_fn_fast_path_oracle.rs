//! Behavior oracle for the `parse_jest_fn_call` fast path.
//!
//! `parse_jest_fn_call` is consumed by an open set of Jest/Vitest rules, several
//! of which run on NON-test files and read its result for arbitrary call names.
//! A previous attempt changed the helper's *return value* on the `(false,false)`
//! framework arm and silently dropped diagnostics for two such consumers
//! (`prefer-importing-jest-globals` for `pending()`, and
//! `prefer-called-exactly-once-with` for a `setTimeout(...)` wrapper).
//!
//! This oracle pins those exact behaviors by driving the real `Linter::run` with
//! every built-in rule enabled, on a NON-test file (no jest/vitest import, plain
//! `.ts` path) so `is_jest()`/`is_vitest()` are both `false` — the arm the fast
//! path must keep byte-identical. Any consumer regression flips one of these
//! assertions.

use std::{path::Path, sync::Arc};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ContextSubHostOptions, ExternalPluginStore,
    FixKind, LintOptions, Linter, ModuleRecord,
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

fn lint_all_rules(source: &str, path: &str) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new_linter().build(&parser_ret.program).semantic;
    let path = Path::new(path);
    let module_record = Arc::new(ModuleRecord::new(path, &parser_ret.module_record, &semantic));
    let mut external_plugin_store = ExternalPluginStore::default();
    let lint_config = ConfigStoreBuilder::all().build(&mut external_plugin_store).unwrap();
    let linter = Linter::new(
        LintOptions::default(),
        ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
        None,
    )
    .with_fix(FixKind::All);
    linter
        .run(
            path,
            vec![ContextSubHost::new(
                semantic,
                Arc::clone(&module_record),
                0,
                ContextSubHostOptions::default(),
            )],
            &allocator,
        )
        .into_iter()
        .map(|message| format!("{}", message.error))
        .collect()
}

/// `prefer-importing-jest-globals` reads only the *name* of an `Unknown`-kind
/// jest call; `pending` is a `JEST_METHOD_NAMES` global with no `JestFnKind`.
#[test]
fn pending_global_in_non_test_file_is_reported() {
    let diagnostics = lint_all_rules("pending();\n", "src/app.ts");
    assert!(
        diagnostics.iter().any(|m| m.contains("@jest/globals") && m.contains("pending")),
        "expected prefer-importing-jest-globals to report `pending`; got: {diagnostics:#?}",
    );
}

/// `prefer-called-exactly-once-with` matches `Some(GeneralJest(_))` for ANY kind
/// (members unused) and recurses into the callback via the call expression, so a
/// non-jest wrapper around the matcher pair must still be analyzed.
#[test]
fn settimeout_wrapper_in_non_test_file_is_reported() {
    let source = "setTimeout(() => {\n  expect(x).toHaveBeenCalledOnce();\n  expect(x).toHaveBeenCalledWith('hoge');\n});\n";
    let diagnostics = lint_all_rules(source, "src/app.ts");
    assert!(
        diagnostics.iter().any(|m| m.contains("toHaveBeenCalledExactlyOnceWith")),
        "expected prefer-called-exactly-once-with to fire; got: {diagnostics:#?}",
    );
}

/// A genuinely non-Jest bare call must never be reported as a Jest global.
#[test]
fn plain_non_jest_call_is_not_reported_as_jest_global() {
    let diagnostics = lint_all_rules("foo();\n", "src/app.ts");
    assert!(
        !diagnostics.iter().any(|m| m.contains("@jest/globals")),
        "did not expect a jest-globals import suggestion for `foo()`; got: {diagnostics:#?}",
    );
}

/// On a Vitest file (`is_vitest() == true`) the slow path rejects an unknown
/// root such as `setTimeout` via `is_valid_vitest_call`, so
/// `prefer-called-exactly-once-with` does NOT fire. The fast path is gated off on
/// test files to preserve this — a naive fast path returned `Some` here and
/// wrongly ADDED a `toHaveBeenCalledExactlyOnceWith` diagnostic.
#[test]
fn settimeout_wrapper_in_vitest_file_is_not_reported() {
    let source = "import { test, expect } from 'vitest';\nsetTimeout(() => {\n  expect(x).toHaveBeenCalledOnce();\n  expect(x).toHaveBeenCalledWith('hoge');\n});\n";
    let diagnostics = lint_all_rules(source, "src/app.ts");
    assert!(
        !diagnostics.iter().any(|m| m.contains("toHaveBeenCalledExactlyOnceWith")),
        "prefer-called-exactly-once-with must NOT fire for a `setTimeout` wrapper on a Vitest file; got: {diagnostics:#?}",
    );
}

/// Same divergent path reached via a `*.test.ts` filename (`is_jest() == true`
/// through `is_jestlike_file`): the unknown root `setTimeout` is rejected by the
/// slow path, so no `toHaveBeenCalledExactlyOnceWith` diagnostic is produced.
#[test]
fn settimeout_wrapper_in_dot_test_file_is_not_reported() {
    let source = "setTimeout(() => {\n  expect(x).toHaveBeenCalledOnce();\n  expect(x).toHaveBeenCalledWith('hoge');\n});\n";
    let diagnostics = lint_all_rules(source, "src/app.test.ts");
    assert!(
        !diagnostics.iter().any(|m| m.contains("toHaveBeenCalledExactlyOnceWith")),
        "prefer-called-exactly-once-with must NOT fire for a `setTimeout` wrapper on a *.test.ts file; got: {diagnostics:#?}",
    );
}
