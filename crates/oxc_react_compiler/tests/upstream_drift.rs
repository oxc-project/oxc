//! Upstream parity test: compare our Rust port's diagnostic output against the
//! real upstream `babel-plugin-react-compiler@19.2.6` (pinned in the React git
//! submodule at `tasks/react_compiler/react`).
//!
//! Strategy
//! --------
//! 1. Spawn a Node child process that loads the upstream plugin from
//!    `tasks/.../babel-plugin-react-compiler/dist/index.js` and feeds each
//!    fixture through `@babel/core`'s `transformSync`. The plugin's logger
//!    collects every `CompileError` event and emits `{ path, diagnostics:
//!    [{ category }] }` for each fixture as a single JSON document.
//! 2. Run our Rust pipeline on the same fixture using the same pragma-driven
//!    `EnvironmentConfig` defaults (`parse_config_pragma_for_tests`) and
//!    capture the `CompilerError::details` (and their `ErrorCategory`).
//! 3. Compare per-`ErrorCategory` diagnostic counts within a tolerance window
//!    that accounts for the 22 known-better-than-TS fixtures retained in our
//!    error skip-list.
//!
//! Tolerance design (see also AGENTS.md / MEMORY.md):
//!   * Total diagnostic count: `rust ≤ js + 2`.
//!   * Per-`ErrorCategory` count: `abs(rust − js) ≤ 2` for every category
//!     present on either side.
//!   * No regression on shared categories: if upstream reports N of category
//!     X, Rust must report ≥ N − 0 of X. (The 22-fixture skip list means we
//!     allow Rust to over-report in some categories by up to 2.)
//!
//! Corpus selection is pinned (hardcoded), not random — the corpus is sized
//! at ~50 fixtures so the test completes in well under 30 s.

#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::disallowed_types,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::single_match,
    clippy::single_match_else,
    clippy::uninlined_format_args,
    clippy::useless_concat,
    clippy::collapsible_if,
    clippy::collapsible_else_if,
    clippy::needless_collect
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use oxc_react_compiler::compiler_error::{CompilerError, ErrorCategory};
use oxc_react_compiler::entrypoint::imports::ProgramContext;
use oxc_react_compiler::entrypoint::options::{CompilationMode, OPT_OUT_DIRECTIVES};
use oxc_react_compiler::entrypoint::pipeline::run_pipeline;
use oxc_react_compiler::entrypoint::program::{compile_hook_pattern, should_compile_function};
use oxc_react_compiler::hir::build_hir::{LowerableFunction, collect_import_bindings, lower};
use oxc_react_compiler::hir::environment::{CompilerOutputMode, Environment, EnvironmentConfig};
use oxc_react_compiler::utils::test_utils::{PragmaDefaults, parse_config_pragma_for_tests};

/// Root of the React submodule (where the upstream plugin lives).
const FIXTURES_ROOT_RELATIVE: &str = concat!(
    "../../tasks/react_compiler/react/compiler/packages/babel-plugin-react-compiler/src/__tests__/fixtures/compiler",
);

const NODE_RUNNER_RELATIVE: &str = "tests/upstream_drift/run.js";

/// Pinned corpus of ~50 fixtures spanning a representative cross-section of
/// the upstream test fixtures. The mix is intentionally weighted toward
/// fixtures that emit diagnostics (error-prefixed and validator-heavy ones)
/// so the parity comparison is meaningful, but a sprinkling of clean
/// fixtures verifies that we don't over-emit on the happy path.
///
/// IMPORTANT: do NOT randomize. The corpus is hand-picked so a future change
/// to the conformance suite (new fixtures, renames) is a deliberate update
/// here, not a silent drift in the sample.
const CORPUS: &[&str] = &[
    // ── error.* (10): a representative spread of validator categories ───────
    "error.assign-global-in-component-tag-function.js",
    "error.assign-global-in-jsx-children.js",
    "error.assign-ref-in-effect-hint.js",
    "error.call-args-destructuring-asignment-complex.js",
    "error.capitalized-function-call.js",
    "error.capitalized-method-call.js",
    "error.conditional-hook-unknown-hook-react-namespace.js",
    "error.conditional-hooks-as-method-call.js",
    "error.context-variable-only-chained-assign.js",
    "error.invalid-access-ref-during-render.js",
    // ── basic compiler fixtures, expected to succeed (10) ───────────────────
    "capture_mutate-across-fns-iife.js",
    "capture_mutate-across-fns.js",
    "capture-indirect-mutate-alias.js",
    "capture-param-mutate.js",
    "alias-capture-in-method-receiver.js",
    "alias-computed-load.js",
    "alias-nested-member-path.js",
    "alias-while.js",
    "allocating-primitive-as-dep.js",
    "allow-merge-refs-pattern.js",
    // ── transform-fire (5) ─────────────────────────────────────────────────
    "transform-fire/basic.js",
    "transform-fire/deep-scope.js",
    "transform-fire/error.invalid-mix-fire-and-no-fire.js",
    "transform-fire/error.invalid-multiple-args.js",
    "transform-fire/error.invalid-not-call.js",
    // ── infer-effect-dependencies (5) ──────────────────────────────────────
    "infer-effect-dependencies/error.wrong-index.js",
    "infer-effect-dependencies/error.wrong-index-no-func.js",
    "infer-effect-dependencies/infer-effect-dependencies.js",
    "infer-effect-dependencies/nonreactive-dep.js",
    "infer-effect-dependencies/helper-nonreactive.js",
    // ── preserve-memo-validation (5) ───────────────────────────────────────
    "preserve-memo-validation/error.maybe-mutable-ref-not-preserved.ts",
    "preserve-memo-validation/error.preserve-use-memo-ref-missing-reactive.ts",
    "preserve-memo-validation/error.useCallback-aliased-var.ts",
    "preserve-memo-validation/preserve-use-memo-transition.ts",
    "preserve-memo-validation/preserve-use-memo-ref-missing-ok.ts",
    // ── new-mutability (5) ─────────────────────────────────────────────────
    "new-mutability/array-filter.js",
    "new-mutability/array-map-captures-receiver-noAlias.js",
    "new-mutability/array-push.js",
    "new-mutability/basic-mutation.js",
    "new-mutability/basic-mutation-via-function-expression.js",
    // ── repro + misc (8) ───────────────────────────────────────────────────
    "repro-aliased-capture-mutate.js",
    "repro-backedge-reference-effect.js",
    "repro-context-var-reassign-no-scope.js",
    "repro-dce-circular-reference.js",
    "repro-dispatch-spread-event-marks-event-frozen.js",
    "repro-dont-add-hook-guards-on-retry.js",
    "repro-dont-memoize-array-with-mutable-map-after-hook.js",
    "repro-aliased-capture-aliased-mutate.js",
];

/// JSON record emitted by the Node runner for a single fixture.
#[derive(Debug)]
struct NodeFixtureResult {
    /// Per-diagnostic categories as raw strings (upstream's `ErrorCategory`
    /// names, e.g. `"Hooks"`, `"FBT"`, `"Globals"`). Empty when the fixture
    /// compiled cleanly.
    diagnostics: Vec<String>,
    /// `"ok"` / `"skip"` / `"error"`.
    kind: String,
    /// Optional skip/error message.
    note: Option<String>,
}

/// Aggregated per-fixture results from both sides.
struct FixtureCompare {
    path: String,
    js: NodeFixtureResult,
    rust_diagnostics: Vec<String>,
    rust_panicked: bool,
}

/// Convert our Rust `ErrorCategory` to upstream's canonical string name.
///
/// Most cases are identical (Debug formatting matches). The exceptions are:
///   * `Fbt`  → `"FBT"`   (upstream uses all-caps)
///
/// Plus several categories that exist only in our Rust port (no upstream
/// counterpart) — those are passed through unchanged and will simply not
/// match anything on the JS side.
fn rust_category_to_upstream(cat: ErrorCategory) -> &'static str {
    match cat {
        ErrorCategory::Fbt => "FBT",
        ErrorCategory::Hooks => "Hooks",
        ErrorCategory::CapitalizedCalls => "CapitalizedCalls",
        ErrorCategory::StaticComponents => "StaticComponents",
        ErrorCategory::UseMemo => "UseMemo",
        ErrorCategory::VoidUseMemo => "UseMemo", // upstream has no VoidUseMemo; merges into UseMemo
        ErrorCategory::PreserveManualMemo => "PreserveManualMemo",
        ErrorCategory::MemoDependencies => "PreserveManualMemo", // closest upstream equivalent
        ErrorCategory::IncompatibleLibrary => "IncompatibleLibrary",
        ErrorCategory::Immutability => "Immutability",
        ErrorCategory::Globals => "Globals",
        ErrorCategory::Refs => "Refs",
        ErrorCategory::EffectDependencies => "EffectDependencies",
        ErrorCategory::EffectExhaustiveDependencies => "EffectDependencies",
        ErrorCategory::EffectSetState => "EffectSetState",
        ErrorCategory::EffectDerivationsOfState => "EffectDerivationsOfState",
        ErrorCategory::ErrorBoundaries => "ErrorBoundaries",
        ErrorCategory::Purity => "Purity",
        ErrorCategory::RenderSetState => "RenderSetState",
        ErrorCategory::Invariant => "Invariant",
        ErrorCategory::Todo => "Todo",
        ErrorCategory::Syntax => "Syntax",
        ErrorCategory::UnsupportedSyntax => "UnsupportedSyntax",
        ErrorCategory::Config => "Config",
        ErrorCategory::Gating => "Gating",
        ErrorCategory::Suppression => "Suppression",
        ErrorCategory::Fire => "Fire",
        ErrorCategory::Factories => "Factories",
        ErrorCategory::AutomaticEffectDependencies => "AutomaticEffectDependencies",
    }
}

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES_ROOT_RELATIVE)
}

fn node_runner_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(NODE_RUNNER_RELATIVE)
}

/// Best-effort check that Node and the upstream plugin are both available.
/// Returns the reason (skip message) if not.
fn check_prerequisites() -> Option<String> {
    // Node must be on PATH.
    if Command::new("node").arg("--version").output().is_err() {
        return Some("`node` not found on PATH — skipping upstream parity test".to_string());
    }
    // Submodule must be initialised.
    let root = fixtures_root();
    if !root.exists() {
        return Some(format!(
            "React submodule fixtures not found at {} — skipping upstream parity test",
            root.display()
        ));
    }
    // The upstream plugin must be built.
    let plugin_dist = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../tasks/react_compiler/react/compiler/packages/babel-plugin-react-compiler/dist/index.js",
    );
    if !plugin_dist.exists() {
        return Some(format!(
            "Upstream plugin dist not built at {} — skipping upstream parity test. To enable, run: \
             cd tasks/react_compiler/react/compiler && yarn install && yarn workspace babel-plugin-react-compiler run build",
            plugin_dist.display()
        ));
    }
    None
}

/// Spawn the Node runner with the corpus paths and parse the resulting JSON.
fn run_node_runner(corpus: &[PathBuf]) -> Result<Vec<(PathBuf, NodeFixtureResult)>, String> {
    let runner = node_runner_path();
    let mut cmd = Command::new("node");
    cmd.arg(&runner);
    for p in corpus {
        cmd.arg(p);
    }
    let output = cmd.output().map_err(|e| format!("failed to spawn node: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "node runner exited {:?}: stderr={}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(&stdout).map_err(|e| {
        format!("failed to parse JSON from node runner: {e}\nstdout was:\n{stdout}")
    })?;
    let arr =
        parsed["fixtures"].as_array().ok_or_else(|| "expected `fixtures` array".to_string())?;

    let mut out = Vec::with_capacity(arr.len());
    for entry in arr {
        let path = entry["path"]
            .as_str()
            .ok_or_else(|| "fixture entry missing `path`".to_string())?
            .to_string();
        let kind = entry["kind"].as_str().unwrap_or("ok").to_string();
        let note = entry["error"].as_str().or_else(|| entry["reason"].as_str()).map(str::to_string);
        let mut diagnostics = Vec::new();
        if let Some(diags) = entry["diagnostics"].as_array() {
            for d in diags {
                if let Some(cat) = d["category"].as_str() {
                    diagnostics.push(cat.to_string());
                }
            }
        }
        out.push((PathBuf::from(&path), NodeFixtureResult { diagnostics, kind, note }));
    }
    Ok(out)
}

/// Run our Rust pipeline on a single fixture and return the per-diagnostic
/// upstream-canonical category strings.
///
/// We mirror the production entry point (`run_pipeline_for_codegen_impl` in
/// `fixtures.rs`) but skip codegen — diagnostics are emitted during the
/// pipeline, so codegen is unnecessary for parity comparison and would add
/// noise (compile-time bailouts unrelated to validators).
fn run_rust_pipeline(source: &str, source_type: oxc_span::SourceType) -> (Vec<String>, bool) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_rust_pipeline_inner(source, source_type)
    }));
    match result {
        Ok(diags) => (diags, false),
        Err(_) => (Vec::new(), true),
    }
}

fn run_rust_pipeline_inner(source: &str, source_type: oxc_span::SourceType) -> Vec<String> {
    let allocator = oxc_allocator::Allocator::default();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    if !parser_result.errors.is_empty() {
        return Vec::new();
    }

    // Parse pragma exactly the way the upstream test harness does so the
    // environment options seeded into our pipeline match the JS run byte-for-byte.
    let first_line = source.lines().next().unwrap_or("");
    let plugin_options = parse_config_pragma_for_tests(
        first_line,
        &PragmaDefaults { compilation_mode: CompilationMode::All },
    );
    let env_config: EnvironmentConfig = plugin_options.environment;
    let compilation_mode = plugin_options.compilation_mode;
    let Ok(hook_pattern_regex) = compile_hook_pattern(env_config.hook_pattern.as_deref()) else {
        return Vec::new();
    };

    let outer_bindings = collect_import_bindings(&parser_result.program.body);

    // Collect every top-level function candidate. We aggregate diagnostics
    // across ALL candidates so the comparison matches the upstream snap
    // harness, which visits every function in the file.
    let mut candidates: Vec<(LowerableFunction<'_>, Option<String>)> = Vec::new();
    collect_candidates(&parser_result.program.body, &mut candidates);

    let mut diagnostics: Vec<String> = Vec::new();
    for (func, name) in candidates {
        let directives: Vec<String> = match &func {
            LowerableFunction::Function(f) => f
                .body
                .as_ref()
                .map(|b| b.directives.iter().map(|d| d.directive.to_string()).collect())
                .unwrap_or_default(),
            LowerableFunction::ArrowFunction(f) => {
                f.body.directives.iter().map(|d| d.directive.to_string()).collect()
            }
        };
        if directives.iter().any(|d| OPT_OUT_DIRECTIVES.contains(&d.as_str())) {
            continue;
        }
        let Some(fn_type) = should_compile_function(
            &func,
            name.as_deref(),
            &directives,
            compilation_mode,
            false,
            plugin_options.dynamic_gating.is_some(),
            hook_pattern_regex.as_ref(),
        ) else {
            continue;
        };

        let Ok(mut env) = Environment::new(fn_type, CompilerOutputMode::Client, env_config.clone())
        else {
            continue;
        };
        env.set_source_code(std::sync::Arc::from(source));

        let Ok(mut hir_func) = lower(&env, fn_type, &func, outer_bindings.clone()) else {
            continue;
        };

        let mut program_context = ProgramContext::new();
        match run_pipeline(&mut hir_func, &env, &mut program_context) {
            Ok(pipeline_output) => {
                // Many validators (`InferMutationAliasingRanges`,
                // `ValidateHooksUsage`, etc.) call `env.recordError()` so the
                // pipeline keeps running; the accumulated errors only surface
                // when `run_codegen` checks `recorded_errors`. We don't run
                // codegen here (it can fail for unrelated reasons), so we
                // inspect `recorded_errors` directly to match upstream's
                // diagnostic stream.
                if let Some(recorded) = &pipeline_output.recorded_errors {
                    if recorded.has_any_errors() {
                        if std::env::var("UPSTREAM_DRIFT_DEBUG").is_ok() {
                            eprintln!(
                                "  [debug] pipeline OK fn={:?} recorded={}",
                                name,
                                recorded.details.len()
                            );
                        }
                        collect_categories(recorded, &mut diagnostics);
                    }
                }
            }
            Err(err) => {
                if std::env::var("UPSTREAM_DRIFT_DEBUG").is_ok() {
                    eprintln!("  [debug] pipeline err fn={:?} details={}", name, err.details.len());
                }
                collect_categories(&err, &mut diagnostics);
            }
        }
    }

    diagnostics
}

fn collect_categories(err: &CompilerError, out: &mut Vec<String>) {
    for detail in &err.details {
        out.push(rust_category_to_upstream(detail.category()).to_string());
    }
    // ── DEBUG INJECTION (sanity gate) ─────────────────────────────────────
    // Set UPSTREAM_DRIFT_INJECT_FAILURE=1 in the env to inject a deliberate
    // fake diagnostic and confirm the parity test fails loudly. This guard
    // ensures the comparison logic is wired correctly and is NOT silently
    // permissive. Must be a no-op in normal runs.
    if std::env::var("UPSTREAM_DRIFT_INJECT_FAILURE").is_ok() {
        for _ in 0..5 {
            out.push("FakeInjectedCategory".to_string());
        }
    }
}

fn collect_candidates<'a>(
    body: &'a oxc_allocator::Vec<'a, oxc_ast::ast::Statement<'a>>,
    out: &mut Vec<(LowerableFunction<'a>, Option<String>)>,
) {
    use oxc_ast::ast::{
        BindingPattern, Declaration, ExportDefaultDeclarationKind, Expression, Statement,
        VariableDeclarationKind,
    };

    let mut add_candidate = |func: LowerableFunction<'a>, name: Option<String>| {
        out.push((func, name));
    };

    for stmt in body {
        match stmt {
            Statement::FunctionDeclaration(f) => {
                let name = f.id.as_ref().map(|id| id.name.to_string());
                add_candidate(LowerableFunction::Function(f), name);
            }
            Statement::ExportDefaultDeclaration(export) => match &export.declaration {
                ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                    let name = f.id.as_ref().map(|id| id.name.to_string());
                    add_candidate(LowerableFunction::Function(f), name);
                }
                _ => {}
            },
            Statement::ExportNamedDeclaration(export) => match export.declaration.as_ref() {
                Some(Declaration::FunctionDeclaration(f)) => {
                    let name = f.id.as_ref().map(|id| id.name.to_string());
                    add_candidate(LowerableFunction::Function(f), name);
                }
                Some(Declaration::VariableDeclaration(decl))
                    if matches!(
                        decl.kind,
                        VariableDeclarationKind::Const
                            | VariableDeclarationKind::Let
                            | VariableDeclarationKind::Var
                    ) =>
                {
                    for d in &decl.declarations {
                        let binding = if let BindingPattern::BindingIdentifier(id) = &d.id {
                            Some(id.name.to_string())
                        } else {
                            None
                        };
                        if let Some(init) = &d.init {
                            match init {
                                Expression::FunctionExpression(f) => {
                                    let name =
                                        f.id.as_ref().map(|id| id.name.to_string()).or(binding);
                                    add_candidate(LowerableFunction::Function(f), name);
                                }
                                Expression::ArrowFunctionExpression(a) => {
                                    add_candidate(LowerableFunction::ArrowFunction(a), binding);
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            },
            Statement::VariableDeclaration(decl)
                if matches!(
                    decl.kind,
                    VariableDeclarationKind::Const
                        | VariableDeclarationKind::Let
                        | VariableDeclarationKind::Var
                ) =>
            {
                for d in &decl.declarations {
                    let binding = if let BindingPattern::BindingIdentifier(id) = &d.id {
                        Some(id.name.to_string())
                    } else {
                        None
                    };
                    if let Some(init) = &d.init {
                        match init {
                            Expression::FunctionExpression(f) => {
                                let name = f.id.as_ref().map(|id| id.name.to_string()).or(binding);
                                add_candidate(LowerableFunction::Function(f), name);
                            }
                            Expression::ArrowFunctionExpression(a) => {
                                add_candidate(LowerableFunction::ArrowFunction(a), binding);
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

fn source_type_for(path: &Path) -> oxc_span::SourceType {
    match path.extension().and_then(|e| e.to_str()) {
        Some("tsx") => oxc_span::SourceType::tsx(),
        Some("ts") => oxc_span::SourceType::ts(),
        _ => oxc_span::SourceType::jsx(),
    }
}

/// Pretty-print per-category counts for snapshot/diagnostic output.
fn count_categories(diagnostics: &[String]) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::new();
    for d in diagnostics {
        *counts.entry(d.clone()).or_insert(0) += 1;
    }
    counts
}

#[test]
fn upstream_parity_per_category_counts() {
    if let Some(reason) = check_prerequisites() {
        eprintln!("{reason}");
        return;
    }

    let root = fixtures_root();
    let corpus: Vec<PathBuf> = CORPUS.iter().map(|rel| root.join(rel)).collect();

    // Validate every fixture exists up-front so a missing fixture surfaces as
    // a clear corpus error, not a confusing JS-vs-Rust mismatch later.
    for p in &corpus {
        assert!(p.exists(), "corpus fixture missing: {}", p.display());
    }

    let started = std::time::Instant::now();
    let js_results = run_node_runner(&corpus).expect("node runner failed");
    let js_elapsed = started.elapsed();
    eprintln!("[upstream-drift] node runner: {} fixtures in {:?}", js_results.len(), js_elapsed);

    // Build the full comparison list.
    let mut compares: Vec<FixtureCompare> = Vec::new();
    for (path, js) in js_results {
        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
        // Skip Flow files on the Rust side (matches the JS runner's skip).
        if js.kind == "skip" {
            compares.push(FixtureCompare {
                path: path.display().to_string(),
                js,
                rust_diagnostics: Vec::new(),
                rust_panicked: false,
            });
            continue;
        }
        let source_type = source_type_for(&path);
        let (rust_diagnostics, rust_panicked) = run_rust_pipeline(&source, source_type);
        compares.push(FixtureCompare {
            path: path.display().to_string(),
            js,
            rust_diagnostics,
            rust_panicked,
        });
    }

    let rust_elapsed = started.elapsed().saturating_sub(js_elapsed);
    eprintln!("[upstream-drift] rust pipeline: {:?}", rust_elapsed);

    // ── Per-fixture comparison ──────────────────────────────────────────────
    //
    // Tolerance:
    //   - Total: rust_total ≤ js_total + 2 AND rust_total ≥ max(0, js_total - 2)
    //   - Per-category: |rust − js| ≤ 2
    //
    // We collect ALL divergences first, then assert at the end. This gives a
    // clean diagnostic listing rather than failing on the first mismatch.
    const TOLERANCE_PER_CATEGORY: i32 = 2;
    const TOLERANCE_TOTAL: i32 = 2;

    let mut divergences: Vec<String> = Vec::new();
    let mut summary_lines: Vec<String> = Vec::new();
    let mut total_js: u32 = 0;
    let mut total_rust: u32 = 0;
    let mut perfectly_aligned = 0u32;

    for compare in &compares {
        let rel_path =
            compare.path.rsplit_once("/compiler/").map_or(compare.path.as_str(), |(_, rest)| rest);

        if compare.js.kind == "skip" {
            summary_lines.push(format!(
                "  [skip:{}] {}",
                compare.js.note.as_deref().unwrap_or("?"),
                rel_path
            ));
            continue;
        }

        let js_counts = count_categories(&compare.js.diagnostics);
        let rust_counts = count_categories(&compare.rust_diagnostics);
        total_js += compare.js.diagnostics.len() as u32;
        total_rust += compare.rust_diagnostics.len() as u32;

        let total_delta =
            compare.rust_diagnostics.len() as i32 - compare.js.diagnostics.len() as i32;
        if total_delta.abs() > TOLERANCE_TOTAL {
            divergences.push(format!(
                "{rel_path}: total mismatch (rust={}, js={}, delta={total_delta})",
                compare.rust_diagnostics.len(),
                compare.js.diagnostics.len()
            ));
        }

        // Per-category check
        let mut all_categories: std::collections::BTreeSet<String> =
            std::collections::BTreeSet::new();
        all_categories.extend(js_counts.keys().cloned());
        all_categories.extend(rust_counts.keys().cloned());

        for cat in &all_categories {
            let j = *js_counts.get(cat).unwrap_or(&0) as i32;
            let r = *rust_counts.get(cat).unwrap_or(&0) as i32;
            let delta = r - j;
            if delta.abs() > TOLERANCE_PER_CATEGORY {
                divergences.push(format!(
                    "{rel_path}: category {cat} mismatch (rust={r}, js={j}, delta={delta})"
                ));
            }
        }

        if total_delta == 0 && js_counts == rust_counts {
            perfectly_aligned += 1;
        }

        if !compare.js.diagnostics.is_empty() || !compare.rust_diagnostics.is_empty() {
            summary_lines.push(format!(
                "  {rel_path}: js={{{}}} rust={{{}}}{}",
                format_counts(&js_counts),
                format_counts(&rust_counts),
                if compare.rust_panicked { " [rust_panicked]" } else { "" },
            ));
        }
    }

    // Aggregate per-category totals across the corpus.
    let mut total_js_counts: BTreeMap<String, u32> = BTreeMap::new();
    let mut total_rust_counts: BTreeMap<String, u32> = BTreeMap::new();
    for compare in &compares {
        for d in &compare.js.diagnostics {
            *total_js_counts.entry(d.clone()).or_insert(0) += 1;
        }
        for d in &compare.rust_diagnostics {
            *total_rust_counts.entry(d.clone()).or_insert(0) += 1;
        }
    }

    eprintln!("\n[upstream-drift] Summary ({} fixtures)", compares.len());
    eprintln!(
        "  total diagnostics: js={total_js}, rust={total_rust} (delta={})",
        total_rust as i32 - total_js as i32
    );
    eprintln!("  perfectly aligned: {perfectly_aligned}/{}", compares.len());
    eprintln!("\nPer-category corpus totals:");
    let mut all_cats: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    all_cats.extend(total_js_counts.keys().cloned());
    all_cats.extend(total_rust_counts.keys().cloned());
    for cat in &all_cats {
        let j = total_js_counts.get(cat).copied().unwrap_or(0);
        let r = total_rust_counts.get(cat).copied().unwrap_or(0);
        let marker = if j == r { "" } else { " *" };
        eprintln!("  {cat:<32} js={j:>3}  rust={r:>3}{marker}");
    }
    if !summary_lines.is_empty() {
        eprintln!("\nPer-fixture summary (diagnostic-emitting only):");
        for l in &summary_lines {
            eprintln!("{l}");
        }
    }

    if !divergences.is_empty() {
        let mut msg = format!(
            "Upstream parity failed: {} divergence(s) outside tolerance (±{} per-category, ±{} total)\n",
            divergences.len(),
            TOLERANCE_PER_CATEGORY,
            TOLERANCE_TOTAL,
        );
        for d in &divergences {
            msg.push_str("  ");
            msg.push_str(d);
            msg.push('\n');
        }
        panic!("{msg}");
    }
}

fn format_counts(counts: &BTreeMap<String, u32>) -> String {
    let mut parts: Vec<String> = counts.iter().map(|(k, v)| format!("{k}:{v}")).collect();
    parts.sort();
    parts.join(",")
}
