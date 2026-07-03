//! Snapshot tests over the upstream React Compiler fixture corpus.
//!
//! Each input under `fixtures/` (copied from the upstream
//! `babel-plugin-react-compiler/src/__tests__/fixtures/compiler` tree) is parsed,
//! analysed, and run through [`oxc_react_compiler::transform`]. The compiled output
//! plus diagnostics are snapshotted with `insta` into `tests/snapshots/`.
//!
//! Per-fixture options come from the first-line `// @directive` pragmas, mirroring
//! the upstream `snap` runner's `makePluginOptions`: the base config is
//! `compilationMode: all` / `panicThreshold: all_errors`, then each recognised
//! directive overrides a `PluginOptions` field or an `EnvironmentConfig` field
//! (booleans, enums, allow/block lists, and gating/instrumentation configs).
//! Directives oxc doesn't model are ignored (as upstream ignores unknown keys).
//!
//! The snapshots are oxc's *own* golden output (not a comparison against Babel),
//! so any change in compiler behaviour surfaces as a snapshot diff. Regenerate with
//! `cargo insta accept` after reviewing.

use std::{fs, path::Path};

use convert_case::{Case, Casing};
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_react_compiler::react_compiler_hir::environment_config::{
    ExhaustiveEffectDepsMode, ExternalFunctionConfig, InstrumentationConfig,
};
use oxc_react_compiler::{
    DynamicGatingConfig, EnvironmentConfig, GatingConfig, PluginOptions, transform,
};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

#[test]
fn snapshots() {
    let fixtures = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures");
    insta::glob!(fixtures, "**/*.{js,cjs,mjs,ts,cts,mts,jsx,tsx}", |path| {
        let source = fs::read_to_string(path).unwrap();
        let snapshot = run_fixture(&source);
        insta::with_settings!({ prepend_module_to_snapshot => false, snapshot_suffix => "", omit_expression => true }, {
            insta::assert_snapshot!(snapshot_name(path), snapshot);
        });
    });
}

/// Parse, analyse, compile, and render the compiled program + diagnostics.
fn run_fixture(source: &str) -> String {
    let (source_type, options) = parse_pragma(source);

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, source_type).parse();
    let mut program = parsed.program;

    // Surface parse failures instead of silently compiling a recovered/dummy AST,
    // so a fixture that stops parsing shows up as a snapshot change.
    let mut out = String::new();
    if !parsed.diagnostics.is_empty() {
        out.push_str("Parse errors:\n\n");
        for error in &parsed.diagnostics {
            out.push_str(&format!("{error:?}\n"));
        }
        out.push('\n');
    }

    // `transform` borrows a `Semantic` built from the pristine program; scope it so
    // the borrow ends before we swap in the compiled program (matching the example).
    let mut result = {
        let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
        transform(&program, &semantic, &allocator, options)
    };
    if let Some(compiled) = result.program.take() {
        program = compiled;
    }

    if !result.diagnostics.is_empty() {
        out.push_str("Diagnostics:\n\n");
        for diagnostic in &result.diagnostics {
            out.push_str(&format!("{diagnostic:?}\n"));
        }
        out.push('\n');
    }
    if result.changed {
        out.push_str(&Codegen::new().build(&program).code);
    } else {
        out.push_str("No changes.");
    }
    out
}

/// Snapshot name = the fixture path relative to `fixtures/`, extension dropped and
/// `/` replaced with `__` so nested fixtures with the same basename don't collide.
fn snapshot_name(path: &Path) -> String {
    let full = path.to_string_lossy();
    let rel = full.rsplit_once("/fixtures/").map_or(full.as_ref(), |(_, rel)| rel);
    let stem = rel.rsplit_once('.').map_or(rel, |(stem, _ext)| stem);
    stem.split('/').collect::<Vec<_>>().join("__")
}

/// Build the per-fixture `SourceType` + `PluginOptions` from the first-line pragmas.
fn parse_pragma(source: &str) -> (SourceType, PluginOptions) {
    // Upstream `snap` defaults: compile everything, surface every error.
    let mut options = PluginOptions {
        compilation_mode: "all".to_string(),
        panic_threshold: "all_errors".to_string(),
        ..PluginOptions::default()
    };

    let mut is_script = false;
    let first_line = source.lines().next().unwrap_or("");
    // Mirror upstream `splitPragma`: each `@`-delimited entry splits at its first
    // `:` into `key` + raw `value` (the value keeps internal spaces, e.g.
    // `["use todo memo"]`); a colon-less entry is a bare flag whose key is its first
    // space-delimited word (any trailing `word` is discarded, as upstream does).
    for entry in first_line.split('@').skip(1) {
        let entry = entry.trim();
        let (key, value) = match entry.split_once(':') {
            Some((k, v)) => (k, Some(v)),
            None => (entry.split(' ').next().unwrap_or(""), None),
        };
        match key {
            "script" => is_script = true,
            "compilationMode" => {
                if let Some(v) = value {
                    options.compilation_mode = unquote(v);
                }
            }
            "panicThreshold" => {
                if let Some(v) = value {
                    options.panic_threshold = unquote(v);
                }
            }
            "outputMode" => options.output_mode = value.map(unquote),
            // `@gating` has a fixed test default (upstream `testComplexPluginOptionDefaults`).
            "gating" => {
                options.gating = Some(GatingConfig {
                    source: "ReactForgetFeatureFlag".to_string(),
                    import_specifier_name: "isForgetEnabled_Fixtures".to_string(),
                });
            }
            "dynamicGating" => {
                if let Some(source) = value.and_then(json_source) {
                    options.dynamic_gating = Some(DynamicGatingConfig { source });
                }
            }
            "customOptOutDirectives" => {
                if let Some(v) = value {
                    options.custom_opt_out_directives = Some(string_array(v));
                }
            }
            other => set_environment_directive(&mut options.environment, other, value),
        }
    }

    // Upstream parses every (non-Flow) fixture with the TypeScript + JSX plugins,
    // regardless of extension, and as a module unless `@script` — so the emitted
    // runtime import is `import`, not `require`. Flow fixtures are excluded from the
    // corpus (oxc has no Flow parser).
    let source_type = if is_script {
        SourceType::tsx().with_script(true)
    } else {
        SourceType::tsx().with_module(true)
    };
    (source_type, options)
}

/// Strip surrounding double quotes from a directive value (`"annotation"` → `annotation`).
fn unquote(value: &str) -> String {
    value.strip_prefix('"').and_then(|v| v.strip_suffix('"')).unwrap_or(value).to_string()
}

/// Apply an `EnvironmentConfig` directive. Booleans default to `true` for a bare
/// `@key` and clear on `@key:false`; enum / `Option` fields parse their value (or
/// use the upstream test default). Directives with no matching field are ignored,
/// matching the upstream runner.
fn set_environment_directive(env: &mut EnvironmentConfig, camel_key: &str, value: Option<&str>) {
    let on = value != Some("false");
    // Upstream spells this test-only key with a double underscore, which
    // `convert_case` would not fold to the field name.
    let snake = if camel_key == "throwUnknownException__testonly" {
        "throw_unknown_exception_testonly".to_string()
    } else {
        camel_key.to_case(Case::Snake)
    };
    match snake.as_str() {
        "enable_preserve_existing_memoization_guarantees" => {
            env.enable_preserve_existing_memoization_guarantees = on;
        }
        "validate_preserve_existing_memoization_guarantees" => {
            env.validate_preserve_existing_memoization_guarantees = on;
        }
        "validate_exhaustive_memoization_dependencies" => {
            env.validate_exhaustive_memoization_dependencies = on;
        }
        "enable_optional_dependencies" => env.enable_optional_dependencies = on,
        "enable_name_anonymous_functions" => env.enable_name_anonymous_functions = on,
        "validate_hooks_usage" => env.validate_hooks_usage = on,
        "validate_ref_access_during_render" => env.validate_ref_access_during_render = on,
        "validate_no_set_state_in_render" => env.validate_no_set_state_in_render = on,
        "enable_use_keyed_state" => env.enable_use_keyed_state = on,
        "validate_no_set_state_in_effects" => env.validate_no_set_state_in_effects = on,
        "validate_no_derived_computations_in_effects" => {
            env.validate_no_derived_computations_in_effects = on;
        }
        "validate_no_derived_computations_in_effects_exp" => {
            env.validate_no_derived_computations_in_effects_exp = on;
        }
        "validate_no_jsx_in_try_statements" => env.validate_no_jsx_in_try_statements = on,
        "validate_static_components" => env.validate_static_components = on,
        "validate_source_locations" => env.validate_source_locations = on,
        "validate_no_impure_functions_in_render" => {
            env.validate_no_impure_functions_in_render = on;
        }
        "validate_no_freezing_known_mutable_functions" => {
            env.validate_no_freezing_known_mutable_functions = on;
        }
        "enable_assume_hooks_follow_rules_of_react" => {
            env.enable_assume_hooks_follow_rules_of_react = on;
        }
        "enable_transitively_freeze_function_expressions" => {
            env.enable_transitively_freeze_function_expressions = on;
        }
        "enable_function_outlining" => env.enable_function_outlining = on,
        "enable_jsx_outlining" => env.enable_jsx_outlining = on,
        "assert_valid_mutable_ranges" => env.assert_valid_mutable_ranges = on,
        "throw_unknown_exception_testonly" => env.throw_unknown_exception_testonly = on,
        "enable_custom_type_definition_for_reanimated" => {
            env.enable_custom_type_definition_for_reanimated = on;
        }
        "enable_treat_ref_like_identifiers_as_refs" => {
            env.enable_treat_ref_like_identifiers_as_refs = on;
        }
        "enable_treat_set_identifiers_as_state_setters" => {
            env.enable_treat_set_identifiers_as_state_setters = on;
        }
        "validate_no_void_use_memo" => env.validate_no_void_use_memo = on,
        "enable_allow_set_state_from_refs_in_effects" => {
            env.enable_allow_set_state_from_refs_in_effects = on;
        }
        "enable_verbose_no_set_state_in_effect" => env.enable_verbose_no_set_state_in_effect = on,
        "enable_forest" => env.enable_forest = on,

        // Non-boolean fields.
        "validate_exhaustive_effect_dependencies" => {
            env.validate_exhaustive_effect_dependencies = match value.map(unquote).as_deref() {
                Some("missing-only") => ExhaustiveEffectDepsMode::MissingOnly,
                Some("extra-only") => ExhaustiveEffectDepsMode::ExtraOnly,
                _ => ExhaustiveEffectDepsMode::All,
            };
        }
        // `Option<Vec<_>>` allow/block lists — the bare directive is `Some([])`
        // (upstream `testComplexConfigDefaults`), else the parsed JSON array.
        "validate_no_capitalized_calls" => {
            env.validate_no_capitalized_calls = Some(value.map(string_array).unwrap_or_default());
        }
        "validate_blocklisted_imports" => {
            env.validate_blocklisted_imports = Some(value.map(string_array).unwrap_or_default());
        }
        "custom_macros" => {
            if let Some(v) = value {
                // Upstream keeps the segment before the first `.`.
                let head = unquote(v).split('.').next().unwrap_or_default().to_string();
                env.custom_macros = Some(vec![head]);
            }
        }
        // `Option<config>` fields with fixed test defaults (upstream `testComplexConfigDefaults`).
        "enable_emit_instrument_forget" => {
            env.enable_emit_instrument_forget = Some(InstrumentationConfig {
                fn_: external_fn("react-compiler-runtime", "useRenderCounter"),
                gating: Some(external_fn("react-compiler-runtime", "shouldInstrument")),
                global_gating: Some("DEV".to_string()),
            });
        }
        "enable_emit_hook_guards" => {
            env.enable_emit_hook_guards =
                Some(external_fn("react-compiler-runtime", "$dispatcherGuard"));
        }
        _ => {}
    }
}

fn external_fn(source: &str, import_specifier_name: &str) -> ExternalFunctionConfig {
    ExternalFunctionConfig {
        source: source.to_string(),
        import_specifier_name: import_specifier_name.to_string(),
    }
}

/// Quoted strings in order, e.g. `["a","b"]` → `["a", "b"]`, `{"source":"x"}` → `["source", "x"]`.
fn quoted(value: &str) -> Vec<String> {
    value.split('"').skip(1).step_by(2).map(str::to_string).collect()
}

/// Elements of a JSON string array such as `["DangerousImport"]`.
fn string_array(value: &str) -> Vec<String> {
    quoted(value)
}

/// The `source` field of a JSON object such as `{"source":"shared-runtime"}`.
fn json_source(value: &str) -> Option<String> {
    let parts = quoted(value);
    parts.iter().position(|s| s == "source").and_then(|i| parts.get(i + 1).cloned())
}
