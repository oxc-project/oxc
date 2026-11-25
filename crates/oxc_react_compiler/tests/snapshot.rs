//! Snapshot tests over the upstream React Compiler fixture corpus.
//!
//! Each input under `fixtures/` (copied from upstream
//! `babel-plugin-react-compiler/src/__tests__/fixtures/compiler`) is parsed,
//! analysed, and run through [`oxc_react_compiler::compile`]; the compiled output
//! plus diagnostics are snapshotted with `insta` into `tests/snapshots/`.
//!
//! Per-fixture options come from the first-line `// @directive` pragmas, mirroring
//! the upstream `snap` runner: the base config is `compilationMode: all` /
//! `panicThreshold: all_errors`, then each recognised directive overrides a
//! `PluginOptions` or `EnvironmentConfig` field. Unmodelled directives are ignored,
//! as upstream ignores unknown keys.
//!
//! The snapshots are oxc's *own* golden output, so any change in compiler behaviour
//! surfaces as a diff. Regenerate with `cargo insta accept` after reviewing.

use std::{
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
};

use cow_utils::CowUtils;
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_react_compiler::{CompileResult, CompilerOutputMode, compile, lint};
use oxc_semantic::SemanticBuilder;

mod common;

use common::parse_pragma;

#[test]
fn snapshots() {
    // Deeply nested fixtures can overflow libtest's default 2 MiB stack during
    // semantic analysis on s390x. Match the formatter conformance suite's stack size.
    std::thread::Builder::new()
        .stack_size(16 * 1024 * 1024)
        .spawn(run_snapshots)
        .unwrap()
        .join()
        .unwrap();
}

fn run_snapshots() {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures");
    insta::glob!(fixtures, "**/*.{js,cjs,mjs,ts,cts,mts,jsx,tsx}", |path| {
        let source = fs::read_to_string(path).unwrap();
        let source = normalize_newlines(&source);
        let snapshot = run_fixture(&source);
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(snapshot_name(path), snapshot);
        });
    });
}

fn normalize_newlines(source: &str) -> String {
    source.cow_replace("\r\n", "\n").cow_replace('\r', "\n").into_owned()
}

/// Parse, analyse, compile, and render the compiled program + diagnostics, plus any
/// divergence between the read-only `lint` entry point and `compile`.
fn run_fixture(source: &str) -> String {
    let (source_type, options) = parse_pragma(source);
    // In lint output mode the compiler validates without rewriting the program, so
    // `changed` is always false. Upstream's `snap` runner still emits the (unmodified)
    // code in that mode alongside the reported findings, so mirror that here rather
    // than collapsing to "No changes." — otherwise every lint fixture would look like a
    // bail-out even though the compiler ran to completion.
    let lint_mode = CompilerOutputMode::from_opts(&options) == CompilerOutputMode::Lint;

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let mut program = parsed.program;

    let mut out = String::new();
    // Surface parse failures rather than silently compiling a recovered/dummy AST.
    push_diagnostics(&mut out, "Parse errors", parsed.diagnostics.as_slice());

    // `compile` and `lint` both borrow a `Semantic` built from the pristine program;
    // scope the borrow so it ends before the output rewrites the program. `compile`
    // runs first on the untouched allocator so its output stays byte-identical to a
    // compile-only run; `lint` then re-runs the same pipeline read-only so we can
    // cross-check its diagnostics against `compile`'s below.
    let (output, diagnostics, fatal, lint_result) = {
        let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
        let (output, diagnostics, fatal) =
            match compile(&program, &semantic, &allocator, options.clone()) {
                CompileResult::Success { output, diagnostics } => (output, diagnostics, false),
                CompileResult::Fatal { diagnostics } => (None, diagnostics, true),
            };
        let lint_result = lint(&program, &semantic, &allocator, options);
        (output, diagnostics, fatal, lint_result)
    };
    let changed = output.is_some();
    if let Some(output) = output {
        output.transform(&mut program);
    }

    push_diagnostics(&mut out, "Diagnostics", diagnostics.as_slice());
    // Mirror the upstream `snap` runner, which always re-emits the program as
    // `## Code` unless a hard error turns the output into `## Error`. So when the
    // compiler cleanly declines to change anything (e.g. `@expectNothingCompiled`,
    // or a file with no React-like functions), or when a lint-mode run reports
    // findings without rewriting the program, echo the reprinted source rather than
    // the `No changes.` marker. The marker is kept only when a non-lint run is
    // fatal (a parse failure or an escalated compile diagnostic), where upstream
    // emits no code — and echoing a parse-recovered AST would be misleading.
    let clean = parsed.diagnostics.is_empty() && !fatal;
    if changed || clean || lint_mode {
        out.push_str(&Codegen::new().build(&program).code);
    } else {
        out.push_str("No changes.");
    }

    // Cross-check the read-only `lint` entry point against `compile`. Both funnel
    // through the same pipeline, so their diagnostics are identical for almost
    // every fixture. They diverge only where the pipeline gates a validation on lint
    // output mode: `@validateNoSetStateInEffects` / `@validateNoJSXInTryStatements` /
    // `@validateStaticComponents` / `@validateNoDerivedComputationsInEffectsExp` fixtures
    // gain lint-only findings, while `@enableEmitHookGuards` conversely errors only when
    // emitting. Surface any divergence in the snapshot so it stays reviewed rather than
    // drifting silently.
    let transform_body = diagnostics_body(diagnostics.as_slice());
    // `lint` wraps each compiler diagnostic with its category for routing. Compare
    // only the wrapped diagnostic so lint and transform modes remain byte-identical.
    let lint_diagnostics = lint_result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.diagnostic.clone())
        .collect::<Vec<_>>();
    let lint_body = diagnostics_body(&lint_diagnostics);
    // A fatal compile may originate from codegen, which lint intentionally does not
    // run, and may replace diagnostics accumulated by earlier passes. A rewrite
    // bailout can preserve the source while lint continues to report errors;
    // under all_errors, those additional lint diagnostics can also be fatal.
    if !fatal && lint_body != transform_body {
        out.push_str("\n\nLint-mode diagnostics (differ from transform):\n\n");
        out.push_str(if lint_body.is_empty() { "(none)\n" } else { &lint_body });
    }
    if !fatal && lint_result.fatal {
        assert_ne!(lint_body, transform_body, "fatality differs without additional diagnostics");
        out.push_str("\nLint mode is fatal.\n");
    }
    out
}

/// Append a `"{label}:\n\n{diag}\n…\n"` section, or nothing when there are none.
fn push_diagnostics(out: &mut String, label: &str, diagnostics: &[impl std::fmt::Debug]) {
    let body = diagnostics_body(diagnostics);
    if body.is_empty() {
        return;
    }
    out.push_str(label);
    out.push_str(":\n\n");
    out.push_str(&body);
    out.push('\n');
}

/// Render diagnostics as one `{diag:?}` line each, or `""` when there are none.
fn diagnostics_body(diagnostics: &[impl std::fmt::Debug]) -> String {
    let mut body = String::new();
    for diagnostic in diagnostics {
        body.push_str(&format!("{diagnostic:?}\n"));
    }
    body
}

/// Snapshot name = fixture path under `fixtures/`, extension dropped and path
/// separators replaced with `__`, so nested fixtures with the same basename
/// don't collide.
fn snapshot_name(path: &Path) -> String {
    let components = path.components().collect::<Vec<_>>();
    let start = components
        .iter()
        .rposition(|component| {
            matches!(component, Component::Normal(part) if *part == OsStr::new("fixtures"))
        })
        .map_or(0, |index| index + 1);

    let mut rel = PathBuf::new();
    for component in &components[start..] {
        rel.push(component.as_os_str());
    }
    rel.set_extension("");

    rel.components()
        .filter_map(|component| match component {
            Component::Normal(part) => Some(part.to_string_lossy()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("__")
}
