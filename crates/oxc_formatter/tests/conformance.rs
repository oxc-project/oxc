//! Prettier conformance for JS / TS (+ JSX).
//!
//! Compares output against the Prettier suite's `tests/format/{js,jsx,typescript}` snapshots
//! via `oxc_formatter_tests::conformance`; js and ts pin their failure reports with `insta`.
//!
//! Debug a specific test: `PRETTIER_FILTER=<substring> cargo test -p oxc_formatter --test conformance -- --nocapture`

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter::JsFormatOptions;
use oxc_formatter_core::{CoreFormatOptions, FormatOptions as _, LineWidth};
use oxc_formatter_tests::{
    OptionSet,
    conformance::{ConformanceConfig, run_conformance},
};
use oxc_span::SourceType;

#[path = "fixtures/options.rs"]
mod options;
use options::apply_js_options;

const IGNORE: &[&str] = &[
    // Base list copied from Biome, then extended/deduplicated
    // https://github.com/biomejs/biome/blob/cd1c8ec4249e8df8d221393586d664537c9fddb2/crates/biome_formatter_test/src/diff_report.rs#L105
    //
    // Bogus nodes
    "typescript/conformance/classes/constructorDeclarations/constructorParameters/readonlyReadonly.ts",
    "typescript/conformance/parser/ecmascript5/Statements/parserES5ForOfStatement21.ts",
    // Expression syntax: `a?.b = c`
    "js/optional-chaining-assignment/",
    // Experimental syntax: `do {}`
    "js/async-do-expressions/",
    "js/do",
    "jsx/do/",
    // Facebook Translation (fbt) is not supported
    "jsx/fbt/",
    // Experimental syntax: `export X from "mod"`
    "js/export-default/export-default-from",
    "js/export-default/escaped",
    // Experimental syntax: `module <id> {}`
    "js/module-blocks",
    // Experimental syntax: `#[]` and `#{}`
    "js/tuple",
    "js/record",
    "js/arrays/tuple-and-record.js",
    "js/arrows/tuple-and-record.js",
    "js/binary-expressions/tuple-and-record.js",
    "js/class-extends/tuple-and-record.js",
    "js/comments-closure-typecast/tuple-and-record.js",
    "js/comments/tuple-and-record.js",
    "js/function-single-destructuring/tuple-and-record.js",
    "js/method-chain/tuple-and-record.js",
    "jsx/tuple/",
    // Experimental syntax: pipeline operator `|>`
    "js/comments-pipeline-own-line",
    "js/partial-application",
    "js/pipeline-operator",
    // Experimental syntax: `::`
    "js/arrows-bind",
    "js/bind-expressions",
    "js/objects/expression.js",
    "js/no-semi-babylon-extensions/no-semi.js",
    // Experimental syntax: `let { #x: x } = ...`
    "js/destructuring-private-fields",
    // Experimental syntax: `import module`
    "js/import-reflection",
    // Experimental syntax: `throw` expressions
    "js/throw_expressions",
    // Experimental syntax: `import defer` / `import source`
    "js/deferred-import-evaluation",
    "js/source-phase-imports",
    // Babel plugins (mostly experimental syntaxes)
    "js/babel-plugins",
    // Embedded languages in template literals
    "js/comments-closure-typecast/styled-components.js",
    "js/multiparser",
    "typescript/multiparser",
    "typescript/angular-component-examples",
    "js/strings/template-literals.js",
    "js/template-literals/css-prop.js",
    "js/template-literals/styled-components-with-expressions.js",
    "js/template-literals/styled-jsx-with-expressions.js",
    "js/template-literals/styled-jsx.js",
    "js/last-argument-expansion/embed.js",
    "jsx/template/styled-components.js",
    "typescript/as/as-const-embedded.ts",
    // Embedded Angular template
    "typescript/decorators-ts/angular.ts",
    // Syntax recovery
    "typescript/error-recovery/",
    // prettier-ignore
    "js/ignore",
    "typescript/prettier-ignore",
    // range formatting (not whole-file; scattered as both `<root>/range/` dirs and `*range*.js` files)
    "range",
    // IDE cursor
    "cursor",
    // Invalid (some of them are recoverable, though)
    "js/call/invalid",
    // Trailing comma after rest element
    "typescript/trailing-comma/invalid.ts",
    // Invalid modifier combos (`readonly accessor`, optional accessor, ambient initializer)
    "typescript/decorator-auto-accessors/decorator-auto-accessors-abstract-class.ts",
    "typescript/decorator-auto-accessors/decorator-auto-accessors-declare-class.ts",
    "typescript/decorator-auto-accessors/decorator-auto-accessors-mixed-modifiers.ts",
    // Ambiguous await
    "js/top-level-await",
    "jsx/top-level-await",
    "typescript/top-level-await",
    "js/ternaries/parenthesis/await-expression.js",
    // Top-level `await (1)` with no import/export: Prettier always parses `.js` as ESM (await expression),
    // while our unambiguous detection leans to script (call expression), whose output is valid under both
    "js/await/like-call.js",
    // ES5 vs ES6+ identifier: Prettier uses ES5 validation, OXC uses ES6+
    // Characters outside BMP (like U+102A7) are valid ES6+ identifiers but not ES5
    "js/quotes/objects.js",
];

/// Option combinations not supported yet; dropped from the test population entirely.
fn skip_unsupported_options(spec: &OptionSet) -> bool {
    spec.get("experimentalTernaries").and_then(serde_json::Value::as_bool) == Some(true)
}

const JS: ConformanceConfig = ConformanceConfig {
    language: "js",
    fixture_roots: &["js", "jsx"],
    exact_parser: None,
    ignore: IGNORE,
    skip_spec: Some(skip_unsupported_options),
};

const TS: ConformanceConfig = ConformanceConfig {
    language: "ts",
    // There is no `tsx` directory, just check `jsx/` works with TS;
    // `SourceType` variant is derived from each spec file's extension.
    fixture_roots: &["typescript", "jsx"],
    exact_parser: None,
    ignore: IGNORE,
    skip_spec: Some(skip_unsupported_options),
};

fn parse_options(spec: &OptionSet) -> JsFormatOptions {
    let mut options = JsFormatOptions::default();
    // Prettier's default `printWidth` is 80 (oxc defaults to 100); the spec's own
    // `printWidth`/`tabWidth`/`useTabs`/`endOfLine` then override inside `apply_js_options`.
    options.apply_core(CoreFormatOptions {
        line_width: LineWidth::try_from(80).unwrap(),
        ..CoreFormatOptions::default()
    });
    apply_js_options(&mut options, spec);
    options
}

fn format_js(path: &Path, source_text: &str, spec: &OptionSet) -> Option<String> {
    let options = parse_options(spec);
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap();
    let formatted = oxc_formatter::format(&allocator, source_text, source_type, options).ok()?;
    Some(formatted.print().ok()?.into_code())
}

/// Deeply nested fixtures overflow libtest's default 2MiB test-thread stack;
/// run on a dedicated big-stack thread instead.
fn with_big_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new().stack_size(16 * 1024 * 1024).spawn(f).unwrap().join().unwrap()
}

#[test]
fn prettier_conformance_js() {
    let Some(report) = with_big_stack(|| run_conformance(&JS, format_js)) else { return };
    insta::assert_snapshot!("prettier-js", report);
}

#[test]
fn prettier_conformance_ts() {
    let Some(report) = with_big_stack(|| run_conformance(&TS, format_js)) else { return };
    insta::assert_snapshot!("prettier-ts", report);
}
