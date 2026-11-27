//! Regression tests for the `parse_jest_fn_call` fast path.
//!
//! Some Jest/Vitest rules call this helper in ordinary source files, where
//! `is_jest()` and `is_vitest()` are both false. These tests pin the consumers
//! that rely on unknown bare calls still returning `Some`, while test files
//! continue to reject unknown roots such as `setTimeout`.

use std::{borrow::Cow, path::Path, sync::Arc};

use rustc_hash::FxHashMap;

use oxc_allocator::Allocator;
use oxc_linter::{
    ConfigStore, ConfigStoreBuilder, ContextSubHost, ContextSubHostOptions, ExternalPluginStore,
    FixKind, LintFilter, LintFilterKind, LintOptions, LintPlugins, Linter, ModuleRecord,
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

/// `pending` is a Jest global name, but its kind is `Unknown`.
#[test]
fn pending_global_in_non_test_file_is_reported() {
    let diagnostics = lint_all_rules("pending();\n", "src/app.ts");
    assert!(
        diagnostics.iter().any(|m| m.contains("@jest/globals") && m.contains("pending")),
        "expected prefer-importing-jest-globals to report `pending`; got: {diagnostics:#?}",
    );
}

/// Unknown wrappers in non-test files are still traversed by Vitest rules.
#[test]
fn settimeout_wrapper_in_non_test_file_is_reported() {
    let source = "setTimeout(() => {\n  expect(x).toHaveBeenCalledOnce();\n  expect(x).toHaveBeenCalledWith('hoge');\n});\n";
    let diagnostics = lint_all_rules(source, "src/app.ts");
    assert!(
        diagnostics.iter().any(|m| m.contains("toHaveBeenCalledExactlyOnceWith")),
        "expected prefer-called-exactly-once-with to fire; got: {diagnostics:#?}",
    );
}

/// Plain unknown calls must not become Jest global import suggestions.
#[test]
fn plain_non_jest_call_is_not_reported_as_jest_global() {
    let diagnostics = lint_all_rules("foo();\n", "src/app.ts");
    assert!(
        !diagnostics.iter().any(|m| m.contains("@jest/globals")),
        "did not expect a jest-globals import suggestion for `foo()`; got: {diagnostics:#?}",
    );
}

/// In Vitest files, unknown roots are rejected by the slow path.
#[test]
fn settimeout_wrapper_in_vitest_file_is_not_reported() {
    let source = "import { test, expect } from 'vitest';\nsetTimeout(() => {\n  expect(x).toHaveBeenCalledOnce();\n  expect(x).toHaveBeenCalledWith('hoge');\n});\n";
    let diagnostics = lint_all_rules(source, "src/app.ts");
    assert!(
        !diagnostics.iter().any(|m| m.contains("toHaveBeenCalledExactlyOnceWith")),
        "prefer-called-exactly-once-with must NOT fire for a `setTimeout` wrapper on a Vitest file; got: {diagnostics:#?}",
    );
}

/// `*.test.ts` files also use the slow path for unknown roots.
#[test]
fn settimeout_wrapper_in_dot_test_file_is_not_reported() {
    let source = "setTimeout(() => {\n  expect(x).toHaveBeenCalledOnce();\n  expect(x).toHaveBeenCalledWith('hoge');\n});\n";
    let diagnostics = lint_all_rules(source, "src/app.test.ts");
    assert!(
        !diagnostics.iter().any(|m| m.contains("toHaveBeenCalledExactlyOnceWith")),
        "prefer-called-exactly-once-with must NOT fire for a `setTimeout` wrapper on a *.test.ts file; got: {diagnostics:#?}",
    );
}

fn lint_single_rule(
    source: &str,
    path: &str,
    plugins: LintPlugins,
    plugin_name: &'static str,
    rule_name: &'static str,
) -> Vec<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let parser_ret = Parser::new(&allocator, source, source_type).parse();
    let semantic = SemanticBuilder::new_linter().build(&parser_ret.program).semantic;
    let path = Path::new(path);
    let module_record = Arc::new(ModuleRecord::new(path, &parser_ret.module_record, &semantic));
    let mut external_plugin_store = ExternalPluginStore::default();
    let filter = LintFilter::deny(LintFilterKind::Rule(
        Cow::Borrowed(plugin_name),
        Cow::Borrowed(rule_name),
    ));
    let lint_config = ConfigStoreBuilder::empty()
        .with_builtin_plugins(plugins)
        .with_filter(&filter)
        .build(&mut external_plugin_store)
        .unwrap();
    let linter = Linter::new(
        LintOptions::default(),
        ConfigStore::new(lint_config, FxHashMap::default(), external_plugin_store),
        None,
    );
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

/// Vitest plugin with globals (no `import ... from 'vitest'`) on a `*.test.ts` file
/// must recognize Vitest-only modifiers like `.runIf`, `.skipIf`, and `.for`.
#[test]
fn vitest_globals_no_standalone_expect_in_dot_test_file() {
    let valid_cases = [
        r#"test.runIf(true)("foobar", () => { expect(true).toEqual(false); });"#,
        r#"test.skipIf(false)("foobar", () => { expect(true).toEqual(false); });"#,
        r#"test.for([])("foobar", () => { expect(true).toEqual(false); });"#,
        r#"test.concurrent.for([])("foobar", () => { expect(true).toEqual(false); });"#,
        r#"test.concurrent.runIf(true)("foobar", () => { expect(true).toEqual(false); });"#,
        r#"test.for`a | b`("foobar", () => { expect(true).toEqual(false); });"#,
        r#"it.runIf(true)("foobar", () => { expect(true).toEqual(false); });"#,
        r#"it.skipIf(false)("foobar", () => { expect(true).toEqual(false); });"#,
        r#"it.for([])("foobar", () => { expect(true).toEqual(false); });"#,
        r#"describe.runIf(true)("suite", () => { it("t", () => { expect(1).toBe(1); }); });"#,
        r#"describe.skipIf(false)("suite", () => { it("t", () => { expect(1).toBe(1); }); });"#,
        r#"describe.for([])("suite", () => { it("t", () => { expect(1).toBe(1); }); });"#,
    ];

    for source in valid_cases {
        let diagnostics = lint_single_rule(
            source,
            "src/app.test.ts",
            LintPlugins::VITEST,
            "vitest",
            "no-standalone-expect",
        );
        assert!(
            diagnostics.is_empty(),
            "expected no diagnostics for vitest globals case `{source}`, got: {diagnostics:#?}"
        );
    }

    let invalid_cases = [
        r#"test.runIf(true)("foobar", () => { expect(true).toEqual(false); }); expect(1).toBe(1);"#,
        "test.runIf(() => { expect(1).toBe(1); });",
        r#"test.runIf(() => { expect(1).toBe(1); })("foobar", () => {});"#,
        r#"describe.runIf(true)("suite", () => { expect(1).toBe(1); });"#,
    ];

    for source in invalid_cases {
        let diagnostics = lint_single_rule(
            source,
            "src/app.test.ts",
            LintPlugins::VITEST,
            "vitest",
            "no-standalone-expect",
        );
        assert_eq!(
            diagnostics.len(),
            1,
            "expected 1 diagnostic for vitest globals invalid case `{source}`, got: {diagnostics:#?}"
        );
    }
}

/// Jest plugin must NOT accept Vitest-only modifiers like `.runIf`, `.skipIf`, or `.for`.
#[test]
fn jest_plugin_rejects_vitest_only_modifiers_in_dot_test_file() {
    let cases = [
        r#"test.runIf(true)("foobar", () => { expect(true).toEqual(false); });"#,
        r#"test.skipIf(false)("foobar", () => { expect(true).toEqual(false); });"#,
        r#"test.for([])("foobar", () => { expect(true).toEqual(false); });"#,
    ];

    for source in cases {
        let diagnostics = lint_single_rule(
            source,
            "src/app.test.ts",
            LintPlugins::JEST,
            "jest",
            "no-standalone-expect",
        );
        assert_eq!(
            diagnostics.len(),
            1,
            "expected jest/no-standalone-expect to reject `{source}`, got: {diagnostics:#?}"
        );
    }
}
