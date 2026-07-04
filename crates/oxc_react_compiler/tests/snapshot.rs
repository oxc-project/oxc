//! Snapshot tests over the upstream React Compiler fixture corpus.
//!
//! Each input under `fixtures/` (copied from upstream
//! `babel-plugin-react-compiler/src/__tests__/fixtures/compiler`) is parsed,
//! analysed, and run through [`oxc_react_compiler::transform`]; the compiled output
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

use std::{fs, path::Path};

use convert_case::{Case, Casing};
use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_react_compiler::react_compiler_hir::Effect;
use oxc_react_compiler::react_compiler_hir::environment_config::{
    ExhaustiveEffectDepsMode, ExternalFunctionConfig, InstrumentationConfig,
};
use oxc_react_compiler::react_compiler_hir::type_config::{
    BuiltInTypeRef, FunctionTypeConfig, HookTypeConfig, ObjectTypeConfig, TypeConfig,
    TypeReferenceConfig, ValueKind,
};
use oxc_react_compiler::react_compiler_utils::FxIndexMap;
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

    let mut out = String::new();
    // Surface parse failures rather than silently compiling a recovered/dummy AST.
    push_diagnostics(&mut out, "Parse errors", parsed.diagnostics.as_slice());

    // `transform` borrows a `Semantic` built from the pristine program; scope the
    // borrow so it ends before we swap in the compiled program.
    let mut result = {
        let semantic = SemanticBuilder::new().with_build_nodes(true).build(&program).semantic;
        transform(&program, &semantic, &allocator, options)
    };
    if let Some(compiled) = result.program.take() {
        program = compiled;
    }

    push_diagnostics(&mut out, "Diagnostics", result.diagnostics.as_slice());
    // Mirror the upstream `snap` runner, which always re-emits the program as
    // `## Code` unless a hard error turns the output into `## Error`. So when the
    // compiler cleanly declines to change anything (e.g. `@expectNothingCompiled`,
    // or a file with no React-like functions), echo the reprinted source rather
    // than the `No changes.` marker. The marker is kept only when an error was
    // reported (parse failure or a compile diagnostic), where upstream emits no
    // code — and echoing a parse-recovered AST would be misleading.
    let clean = parsed.diagnostics.is_empty() && result.diagnostics.as_slice().is_empty();
    if result.changed || clean {
        out.push_str(&Codegen::new().build(&program).code);
    } else {
        out.push_str("No changes.");
    }
    out
}

/// Append a `"{label}:\n\n{diag}\n…\n"` section, or nothing when there are none.
fn push_diagnostics(out: &mut String, label: &str, diagnostics: &[impl std::fmt::Debug]) {
    if diagnostics.is_empty() {
        return;
    }
    out.push_str(label);
    out.push_str(":\n\n");
    for diagnostic in diagnostics {
        out.push_str(&format!("{diagnostic:?}\n"));
    }
    out.push('\n');
}

/// Snapshot name = fixture path under `fixtures/`, extension dropped and `/` → `__`
/// so nested fixtures with the same basename don't collide.
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
    // Mirror the snap runner's test `moduleTypeProvider`; unlisted modules fall back to
    // oxc's default provider, so only the `error.invalid-{type-provider,known-incompatible}-*`
    // fixtures (which import these magic modules) are affected.
    options.environment.module_type_provider = Some(test_module_type_provider());

    let mut is_script = false;
    // Fixture headers are normalised to the canonical `@key:value` / bare `@key` shape,
    // so each `@`-entry either splits at its first `:` into key + raw value (values keep
    // internal spaces, e.g. `["use todo memo"]`) or is a bare flag.
    for entry in source.lines().next().unwrap_or("").split('@').skip(1) {
        let entry = entry.trim();
        let (key, value) = match entry.split_once(':') {
            Some((key, value)) => (key, Some(value)),
            None => (entry, None),
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
            // Fixed test default (upstream `testComplexPluginOptionDefaults`).
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
                    options.custom_opt_out_directives = Some(json_strings(v));
                }
            }
            other => set_environment_directive(&mut options.environment, other, value),
        }
    }

    // Upstream parses every (non-Flow) fixture with the TypeScript + JSX plugins,
    // regardless of extension, and as a module unless `@script` — so injected runtime
    // imports are `import`, not `require`. Flow fixtures are excluded (no oxc parser).
    let source_type = if is_script {
        SourceType::tsx().with_script(true)
    } else {
        SourceType::tsx().with_module(true)
    };
    (source_type, options)
}

/// Strip surrounding double quotes: `"annotation"` → `annotation`.
fn unquote(value: &str) -> String {
    value.strip_prefix('"').and_then(|v| v.strip_suffix('"')).unwrap_or(value).to_string()
}

/// Apply a non-boolean `EnvironmentConfig` directive, delegating plain booleans to
/// [`set_environment_bool`]. Unmodelled keys are ignored, matching the upstream runner.
fn set_environment_directive(env: &mut EnvironmentConfig, camel_key: &str, value: Option<&str>) {
    // `throwUnknownException__testonly`'s double underscore won't fold via convert_case.
    let snake = if camel_key == "throwUnknownException__testonly" {
        "throw_unknown_exception_testonly".to_string()
    } else {
        camel_key.to_case(Case::Snake)
    };
    match snake.as_str() {
        "validate_exhaustive_effect_dependencies" => {
            env.validate_exhaustive_effect_dependencies = match value.map(unquote).as_deref() {
                Some("missing-only") => ExhaustiveEffectDepsMode::MissingOnly,
                Some("extra-only") => ExhaustiveEffectDepsMode::ExtraOnly,
                _ => ExhaustiveEffectDepsMode::All,
            };
        }
        // Allow/block lists — a bare directive is `Some([])` (upstream
        // `testComplexConfigDefaults`), else the parsed JSON array.
        "validate_no_capitalized_calls" => {
            env.validate_no_capitalized_calls = Some(value.map(json_strings).unwrap_or_default());
        }
        "validate_blocklisted_imports" => {
            env.validate_blocklisted_imports = Some(value.map(json_strings).unwrap_or_default());
        }
        // Upstream keeps only the macro name (before the first `.`).
        "custom_macros" => {
            if let Some(v) = value {
                let name = unquote(v).split('.').next().unwrap_or_default().to_string();
                env.custom_macros = Some(vec![name]);
            }
        }
        // Configs with fixed test defaults (upstream `testComplexConfigDefaults`).
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
        _ => set_environment_bool(env, &snake, value != Some("false")),
    }
}

/// Set a boolean `EnvironmentConfig` field whose name equals the snake-case directive
/// key. A bare `@key` (or `@key:true`) sets it; `@key:false` clears it.
fn set_environment_bool(env: &mut EnvironmentConfig, snake: &str, on: bool) {
    macro_rules! fields {
        ($($field:ident)*) => {
            match snake {
                $(stringify!($field) => env.$field = on,)*
                _ => {}
            }
        };
    }
    fields! {
        enable_preserve_existing_memoization_guarantees
        validate_preserve_existing_memoization_guarantees
        validate_exhaustive_memoization_dependencies
        enable_optional_dependencies
        enable_name_anonymous_functions
        validate_hooks_usage
        validate_ref_access_during_render
        validate_no_set_state_in_render
        enable_use_keyed_state
        validate_no_set_state_in_effects
        validate_no_derived_computations_in_effects
        validate_no_derived_computations_in_effects_exp
        validate_no_jsx_in_try_statements
        validate_static_components
        validate_source_locations
        validate_no_impure_functions_in_render
        validate_no_freezing_known_mutable_functions
        enable_assume_hooks_follow_rules_of_react
        enable_transitively_freeze_function_expressions
        enable_function_outlining
        enable_jsx_outlining
        assert_valid_mutable_ranges
        throw_unknown_exception_testonly
        enable_custom_type_definition_for_reanimated
        enable_treat_ref_like_identifiers_as_refs
        enable_treat_set_identifiers_as_state_setters
        validate_no_void_use_memo
        enable_allow_set_state_from_refs_in_effects
        enable_verbose_no_set_state_in_effect
        enable_forest
    }
}

fn external_fn(source: &str, import_specifier_name: &str) -> ExternalFunctionConfig {
    ExternalFunctionConfig {
        source: source.to_string(),
        import_specifier_name: import_specifier_name.to_string(),
    }
}

/// The double-quoted strings inside a JSON literal, in order:
/// `["a","b"]` → `["a", "b"]`, `{"source":"x"}` → `["source", "x"]`.
fn json_strings(value: &str) -> Vec<String> {
    value.split('"').skip(1).step_by(2).map(str::to_string).collect()
}

/// The `source` field of a JSON object literal like `{"source":"shared-runtime"}`.
fn json_source(value: &str) -> Option<String> {
    let parts = json_strings(value);
    parts.iter().position(|s| s == "source").and_then(|i| parts.get(i + 1).cloned())
}

/// The upstream snap runner's test `moduleTypeProvider` (`shared-runtime-type-provider`),
/// limited to the magic modules the corpus imports to exercise type-config /
/// known-incompatible-library validation. These configs are intentionally invalid or
/// flagged, so the `error.invalid-type-provider-*` / `error.invalid-known-incompatible-*`
/// fixtures produce diagnostics instead of compiling.
fn test_module_type_provider() -> FxIndexMap<String, TypeConfig> {
    FxIndexMap::from_iter([
        (
            "ReactCompilerKnownIncompatibleTest".to_string(),
            object([
                (
                    "useKnownIncompatible",
                    hook(
                        type_ref(),
                        Some(Vec::new()),
                        incompat("useKnownIncompatible is known to be incompatible"),
                    ),
                ),
                (
                    "useKnownIncompatibleIndirect",
                    hook(
                        object([(
                            "incompatible",
                            incompatible_fn(
                                "useKnownIncompatibleIndirect returns an incompatible() function that is known incompatible",
                            ),
                        )]),
                        Some(Vec::new()),
                        None,
                    ),
                ),
                (
                    "knownIncompatible",
                    incompatible_fn("useKnownIncompatible is known to be incompatible"),
                ),
            ]),
        ),
        (
            "ReactCompilerTest".to_string(),
            object([
                ("useHookNotTypedAsHook", type_ref()),
                ("notAhookTypedAsHook", hook(type_ref(), None, None)),
            ]),
        ),
        ("useDefaultExportNotTypedAsHook".to_string(), object([("default", type_ref())])),
    ])
}

fn object(properties: impl IntoIterator<Item = (&'static str, TypeConfig)>) -> TypeConfig {
    let properties = properties.into_iter().map(|(name, config)| (name.to_string(), config));
    TypeConfig::Object(ObjectTypeConfig { properties: Some(properties.collect()) })
}

fn type_ref() -> TypeConfig {
    TypeConfig::TypeReference(TypeReferenceConfig { name: BuiltInTypeRef::Any })
}

fn hook(
    return_type: TypeConfig,
    positional_params: Option<Vec<Effect>>,
    known_incompatible: Option<String>,
) -> TypeConfig {
    let rest_param = positional_params.as_ref().map(|_| Effect::Read);
    TypeConfig::Hook(HookTypeConfig {
        positional_params,
        rest_param,
        return_type: Box::new(return_type),
        return_value_kind: None,
        no_alias: None,
        aliasing: None,
        known_incompatible,
    })
}

fn incompatible_fn(message: &str) -> TypeConfig {
    TypeConfig::Function(FunctionTypeConfig {
        positional_params: Vec::new(),
        rest_param: Some(Effect::Read),
        callee_effect: Effect::Read,
        return_type: Box::new(type_ref()),
        return_value_kind: ValueKind::Mutable,
        no_alias: None,
        mutable_only_if_operands_are_mutable: None,
        impure: None,
        canonical_name: None,
        aliasing: None,
        known_incompatible: Some(message.to_string()),
    })
}

fn incompat(message: &str) -> Option<String> {
    Some(message.to_string())
}
