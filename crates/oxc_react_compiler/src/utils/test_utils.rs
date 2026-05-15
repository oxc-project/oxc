/// Test utilities for the React Compiler.
///
/// Port of `Utils/TestUtils.ts` from the React Compiler.
///
/// Provides utilities for parsing test configuration pragmas from
/// fixture files and setting up the compiler for test execution.
use crate::entrypoint::options::{
    CompilationMode, CompilerReactTarget, DynamicGatingOptions, PanicThreshold, PluginOptions,
};
use crate::hir::environment::{
    EnvironmentConfig, ExhaustiveEffectDepsMode, ExternalFunction, InferEffectDependenciesEntry,
    InlineJsxTransformConfig, InstrumentationConfig,
};

/// Parse a config pragma string from a test fixture's first line.
///
/// Pragma format: `@flag` or `@flag:value`
///
/// Example: `@enableForest @validateNoSetStateInEffects:true @validateNoSetStateInRender:false`
pub fn parse_config_pragma_for_tests(pragma: &str, defaults: &PragmaDefaults) -> PluginOptions {
    let mut options = PluginOptions {
        compilation_mode: defaults.compilation_mode,
        panic_threshold: PanicThreshold::AllErrors,
        ..PluginOptions::default()
    };

    let mut env_config = EnvironmentConfig {
        // Match the TS snap test harness behavior: default to NOT validating
        // preserve-existing-memoization-guarantees.
        validate_preserve_existing_memoization_guarantees: false,
        // Match the TS snap test harness behavior: always register the shared-runtime
        // module type provider.
        enable_shared_runtime_type_provider: true,
        ..EnvironmentConfig::default()
    };

    for entry in split_pragma(pragma) {
        match entry.key.as_str() {
            "enableForest" => {
                env_config.enable_forest = parse_bool_value(entry.value.as_ref(), true);
            }
            "enableFunctionOutlining" => {
                env_config.enable_function_outlining = parse_bool_value(entry.value.as_ref(), true);
            }
            "enableJsxOutlining" => {
                env_config.enable_jsx_outlining = parse_bool_value(entry.value.as_ref(), true);
            }
            "enableNameAnonymousFunctions" => {
                env_config.enable_name_anonymous_functions =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateHooksUsage" => {
                env_config.validate_hooks_usage = parse_bool_value(entry.value.as_ref(), true);
            }
            "validateNoSetStateInRender" => {
                env_config.validate_no_set_state_in_render =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateNoSetStateInEffects" => {
                env_config.validate_no_set_state_in_effects =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateRefAccessDuringRender" => {
                env_config.validate_ref_access_during_render =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateNoDerivedComputationsInEffects" => {
                env_config.validate_no_derived_computations_in_effects =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateNoDerivedComputationsInEffects_exp" => {
                env_config.validate_no_derived_computations_in_effects_exp =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateNoJSXInTryStatements" => {
                env_config.validate_no_jsx_in_try_statements =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateNoImpureFunctionsInRender" => {
                env_config.validate_no_impure_functions_in_render =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateStaticComponents" => {
                env_config.validate_static_components =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateSourceLocations" => {
                env_config.validate_source_locations = parse_bool_value(entry.value.as_ref(), true);
            }
            "assertValidMutableRanges" => {
                env_config.assert_valid_mutable_ranges =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateMemoizedEffectDependencies" => {
                env_config.validate_memoized_effect_dependencies =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validatePreserveExistingMemoizationGuarantees" => {
                env_config.validate_preserve_existing_memoization_guarantees =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enablePreserveExistingMemoizationGuarantees" => {
                env_config.enable_preserve_existing_memoization_guarantees =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableNewMutationAliasingModel" => {
                // Tri-state: bare/`:true` => Some(true), `:false` => Some(false).
                env_config.enable_new_mutation_aliasing_model =
                    Some(parse_bool_value(entry.value.as_ref(), true));
            }
            "enablePropagateDepsInHIR" => {
                env_config.enable_propagate_deps_in_hir =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableFire" => {
                env_config.enable_fire = parse_bool_value(entry.value.as_ref(), true);
            }
            "inferEffectDependencies" => {
                // Bare form (or `:true`) populates with the test-complex defaults
                // from upstream `Utils/TestUtils.ts`:
                //   - react/useEffect           (autodepsIndex: 1)
                //   - shared-runtime/useSpecialEffect (autodepsIndex: 2)
                //   - useEffectWrapper/default  (autodepsIndex: 1)
                // `:false` clears it.
                let enabled = parse_bool_value(entry.value.as_ref(), true);
                env_config.infer_effect_dependencies = if enabled {
                    Some(vec![
                        InferEffectDependenciesEntry {
                            function: ExternalFunction {
                                source: "react".to_string(),
                                import_specifier_name: "useEffect".to_string(),
                            },
                            autodeps_index: 1,
                        },
                        InferEffectDependenciesEntry {
                            function: ExternalFunction {
                                source: "shared-runtime".to_string(),
                                import_specifier_name: "useSpecialEffect".to_string(),
                            },
                            autodeps_index: 2,
                        },
                        InferEffectDependenciesEntry {
                            function: ExternalFunction {
                                source: "useEffectWrapper".to_string(),
                                import_specifier_name: "default".to_string(),
                            },
                            autodeps_index: 1,
                        },
                    ])
                } else {
                    None
                };
            }
            "enableCustomTypeDefinitionForReanimated" => {
                env_config.enable_custom_type_definition_for_reanimated =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableAssumeHooksFollowRulesOfReact" => {
                env_config.enable_assume_hooks_follow_rules_of_react =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableTreatSetIdentifiersAsStateSetters" => {
                env_config.enable_treat_set_identifiers_as_state_setters =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "throwUnknownException__testonly" => {
                env_config.throw_unknown_exception_testonly =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableResetCacheOnSourceFileChanges" => {
                env_config.enable_reset_cache_on_source_file_changes =
                    Some(parse_bool_value(entry.value.as_ref(), true));
            }
            "compilationMode" => {
                if let Some(val) = &entry.value {
                    // Strip surrounding quotes to match TS tryParseTestPragmaValue
                    let stripped = val.trim_matches('"');
                    options.compilation_mode = match stripped {
                        "infer" => CompilationMode::Infer,
                        "syntax" => CompilationMode::Syntax,
                        "annotation" => CompilationMode::Annotation,
                        "all" => CompilationMode::All,
                        _ => defaults.compilation_mode,
                    };
                }
            }
            "panicThreshold" => {
                if let Some(val) = &entry.value {
                    // Strip surrounding quotes to match TS tryParseTestPragmaValue
                    let stripped = val.trim_matches('"');
                    options.panic_threshold = match stripped {
                        "critical_errors" => PanicThreshold::CriticalErrors,
                        "none" => PanicThreshold::None,
                        _ => PanicThreshold::AllErrors,
                    };
                }
            }
            "customMacros" => {
                if let Some(val) = &entry.value {
                    // Strip surrounding quotes, then take the part before the first dot.
                    // Matches TS: `parsedVal.split('.')[0]`
                    let stripped = val.trim_matches('"');
                    let name = stripped.split('.').next().unwrap_or(stripped);
                    if !name.is_empty() {
                        env_config.custom_macros = Some(vec![name.to_string()]);
                    }
                }
            }
            "validateBlocklistedImports" => {
                if let Some(val) = &entry.value
                    && let Some(arr) = parse_json_string_array(val)
                {
                    env_config.validate_blocklisted_imports = Some(arr);
                }
            }
            "enableUseKeyedState" => {
                env_config.enable_use_keyed_state = parse_bool_value(entry.value.as_ref(), true);
            }
            "enableAllowSetStateFromRefsInEffects" => {
                env_config.enable_allow_set_state_from_refs_in_effects =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableVerboseNoSetStateInEffect" => {
                env_config.enable_verbose_no_set_state_in_effect =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableOptionalDependencies" => {
                env_config.enable_optional_dependencies =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableTransitivelyFreezeFunctionExpressions" => {
                env_config.enable_transitively_freeze_function_expressions =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableTreatRefLikeIdentifiersAsRefs" => {
                env_config.enable_treat_ref_like_identifiers_as_refs =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableUseTypeAnnotations" => {
                env_config.enable_use_type_annotations =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableTreatFunctionDepsAsConditional" => {
                env_config.enable_treat_function_deps_as_conditional =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enablePreserveExistingManualUseMemo" => {
                env_config.enable_preserve_existing_manual_use_memo =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableInstructionReordering" => {
                // Upstream `Environment.ts:427`:
                //   enableInstructionReordering: z.boolean().default(false)
                // Pragma forms: `@enableInstructionReordering`,
                // `@enableInstructionReordering:true`,
                // `@enableInstructionReordering:false`.
                env_config.enable_instruction_reordering =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "hookPattern" => {
                // Upstream `Environment.ts:605`:
                //   hookPattern: z.string().nullable().default(null)
                //
                // Pragma form: `@hookPattern:".*\b(use[^$]+)$"`. Strip the
                // surrounding double quotes (matching TS `tryParseTestPragmaValue`).
                if let Some(val) = &entry.value {
                    let stripped = val.trim().trim_matches('"');
                    if stripped.is_empty() || stripped == "null" {
                        env_config.hook_pattern = None;
                    } else {
                        env_config.hook_pattern = Some(stripped.to_string());
                    }
                }
            }
            "validateNoVoidUseMemo" => {
                env_config.validate_no_void_use_memo = parse_bool_value(entry.value.as_ref(), true);
            }
            "enableEmitHookGuards" => {
                // Matches TS testComplexConfigDefaults.enableEmitHookGuards
                env_config.enable_emit_hook_guards = Some(ExternalFunction {
                    source: "react-compiler-runtime".to_string(),
                    import_specifier_name: "$dispatcherGuard".to_string(),
                });
            }
            "enableEmitInstrumentForget" => {
                // Matches TS testComplexConfigDefaults.enableEmitInstrumentForget
                env_config.enable_emit_instrument_forget = Some(InstrumentationConfig {
                    func: ExternalFunction {
                        source: "react-compiler-runtime".to_string(),
                        import_specifier_name: "useRenderCounter".to_string(),
                    },
                    gating: Some(ExternalFunction {
                        source: "react-compiler-runtime".to_string(),
                        import_specifier_name: "shouldInstrument".to_string(),
                    }),
                    global_gating: Some("DEV".to_string()),
                });
            }
            "enableEmitFreeze" => {
                // Matches TS testComplexConfigDefaults.enableEmitFreeze.
                // The pragma is enabled when it has no value, when the value is `true`,
                // or when the value parses to an object via JSON. Setting to `false`
                // disables the feature.
                let value_str = entry.value.as_deref().map(str::trim);
                let disabled = matches!(value_str, Some("false"));
                if !disabled {
                    env_config.enable_emit_freeze = Some(ExternalFunction {
                        source: "react-compiler-runtime".to_string(),
                        import_specifier_name: "makeReadOnly".to_string(),
                    });
                }
            }
            "inlineJsxTransform" => {
                // Matches TS testComplexConfigDefaults.inlineJsxTransform
                // (`Utils/TestUtils.ts` lines 64-67):
                //   { elementSymbol: 'react.transitional.element', globalDevVar: 'DEV' }
                //
                // Inline JSON object values
                // (`@inlineJsxTransform:{"elementSymbol":"...","globalDevVar":"..."}`)
                // override the default. Setting `:false` disables the feature.
                let value_str = entry.value.as_deref().map(str::trim);
                let disabled = matches!(value_str, Some("false"));
                if !disabled {
                    let parsed = entry.value.as_deref().and_then(parse_inline_jsx_transform_value);
                    env_config.inline_jsx_transform =
                        Some(parsed.unwrap_or_else(|| InlineJsxTransformConfig {
                            element_symbol: "react.transitional.element".to_string(),
                            global_dev_var: "DEV".to_string(),
                        }));
                }
            }
            "lowerContextAccess" => {
                // Matches TS testComplexConfigDefaults.lowerContextAccess
                // (`Utils/TestUtils.ts` lines 68-71):
                //   { source: 'react-compiler-runtime',
                //     importSpecifierName: 'useContext_withSelector' }
                //
                // Inline JSON object values
                // (`@lowerContextAccess:{"source":"...","importSpecifierName":"..."}`)
                // override the default. Setting `:false` disables the feature.
                let value_str = entry.value.as_deref().map(str::trim);
                let disabled = matches!(value_str, Some("false"));
                if !disabled {
                    let parsed = entry.value.as_deref().and_then(parse_external_function_value);
                    env_config.lower_context_access =
                        Some(parsed.unwrap_or_else(|| ExternalFunction {
                            source: "react-compiler-runtime".to_string(),
                            import_specifier_name: "useContext_withSelector".to_string(),
                        }));
                }
            }
            "disableMemoizationForDebugging" => {
                // Schema default is `false`. When the pragma is present with no value
                // or `:true`, set to true; `:false` disables it explicitly.
                env_config.disable_memoization_for_debugging =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableChangeVariableCodegen" => {
                // Schema default is `false`. When the pragma is present with no value
                // or `:true`, set to true; `:false` disables it explicitly.
                env_config.enable_change_variable_codegen =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableChangeDetectionForDebugging" => {
                // Matches TS `testComplexConfigDefaults.enableChangeDetectionForDebugging`
                // (`Utils/TestUtils.ts`):
                //   { source: 'react-compiler-runtime', importSpecifierName: '$structuralCheck' }
                //
                // Inline JSON object values (e.g. `:{"source":"...","importSpecifierName":"..."}`)
                // override the default. Setting `:false` is a no-op here because the schema
                // default is already `null`.
                let value_str = entry.value.as_deref().map(str::trim);
                let disabled = matches!(value_str, Some("false"));
                if !disabled {
                    let parsed = entry.value.as_deref().and_then(parse_external_function_value);
                    env_config.enable_change_detection_for_debugging =
                        Some(parsed.unwrap_or_else(|| ExternalFunction {
                            source: "react-compiler-runtime".to_string(),
                            import_specifier_name: "$structuralCheck".to_string(),
                        }));
                }
            }
            "validateNoCapitalizedCalls" => {
                // When the pragma is present, enable the validation.
                // The value is an optional JSON array of allowed function names,
                // e.g. @validateNoCapitalizedCalls:["MyHelper","OtherFunc"]
                let allowlist =
                    entry.value.as_ref().map(|v| parse_string_array(v)).unwrap_or_default();
                env_config.validate_no_capitalized_calls = Some(allowlist);
            }
            "validateExhaustiveMemoizationDependencies" => {
                env_config.validate_exhaustive_memoization_dependencies =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "validateExhaustiveEffectDependencies" => {
                if let Some(val) = &entry.value {
                    let stripped = val.trim_matches('"');
                    env_config.validate_exhaustive_effect_dependencies = match stripped {
                        "missing-only" => ExhaustiveEffectDepsMode::MissingOnly,
                        "extra-only" => ExhaustiveEffectDepsMode::ExtraOnly,
                        "off" => ExhaustiveEffectDepsMode::Off,
                        _ => ExhaustiveEffectDepsMode::All,
                    };
                } else {
                    env_config.validate_exhaustive_effect_dependencies =
                        ExhaustiveEffectDepsMode::All;
                }
            }
            "gating" => {
                if let Some(val) = &entry.value {
                    // @gating:{"source":"shared-runtime","importSpecifierName":"getTrue"}
                    if let Some(ext_fn) = parse_external_function_value(val) {
                        options.gating = Some(ext_fn);
                    }
                } else {
                    // @gating with no value — use the test complex defaults
                    // (matches TS testComplexPluginOptionDefaults.gating)
                    options.gating = Some(ExternalFunction {
                        source: "ReactForgetFeatureFlag".to_string(),
                        import_specifier_name: "isForgetEnabled_Fixtures".to_string(),
                    });
                }
            }
            "eslintSuppressionRules" => {
                if let Some(val) = &entry.value
                    && let Some(arr) = parse_json_string_array(val)
                {
                    options.eslint_suppression_rules = Some(arr);
                }
            }
            "dynamicGating" => {
                if let Some(val) = &entry.value {
                    if let Some(source) = parse_dynamic_gating_value(val) {
                        options.dynamic_gating = Some(DynamicGatingOptions { source });
                    }
                } else {
                    // @dynamicGating with no value — not valid, ignore
                }
            }
            "target" => {
                if let Some(val) = &entry.value {
                    // Strip surrounding quotes to match TS tryParseTestPragmaValue
                    let stripped = val.trim_matches('"');
                    options.target = match stripped {
                        "17" => CompilerReactTarget::React17,
                        "18" => CompilerReactTarget::React18,
                        "donotuse_meta_internal" => CompilerReactTarget::MetaInternal {
                            runtime_module: "react".to_string(),
                        },
                        _ => CompilerReactTarget::React19,
                    };
                }
            }
            _ => {
                // Unknown pragma — ignore
            }
        }
    }

    // Unless explicitly enabled, do not insert HMR handling code
    // in test fixtures or playground to reduce visual noise.
    if env_config.enable_reset_cache_on_source_file_changes.is_none() {
        env_config.enable_reset_cache_on_source_file_changes = Some(false);
    }

    options.environment = env_config;
    options
}

/// Defaults for pragma parsing.
pub struct PragmaDefaults {
    pub compilation_mode: CompilationMode,
}

struct PragmaEntry {
    key: String,
    value: Option<String>,
}

fn split_pragma(pragma: &str) -> Vec<PragmaEntry> {
    let mut entries = Vec::new();
    for entry in pragma.split('@') {
        let key_val = entry.trim();
        if key_val.is_empty() {
            continue;
        }
        let (key, value) = if let Some(idx) = key_val.find(':') {
            let key = key_val[..idx].to_string();
            let val = key_val[idx + 1..].to_string();
            (key, Some(val))
        } else {
            let key = key_val.split_whitespace().next().unwrap_or(key_val).to_string();
            (key, None)
        };
        entries.push(PragmaEntry { key, value });
    }
    entries
}

/// Parse a JSON-like string array from a pragma value, e.g. `["foo","bar"]`.
fn parse_json_string_array(val: &str) -> Option<Vec<String>> {
    let trimmed = val.trim();
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    let inner = inner.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }
    let mut result = Vec::new();
    for item in inner.split(',') {
        let item = item.trim().trim_matches('"');
        if !item.is_empty() {
            result.push(item.to_string());
        }
    }
    Some(result)
}

/// Parse a dynamic gating pragma value, e.g. `{"source":"shared-runtime"}`.
/// Returns the `source` string if successfully parsed.
fn parse_dynamic_gating_value(val: &str) -> Option<String> {
    let trimmed = val.trim();
    let inner = trimmed.strip_prefix('{')?.strip_suffix('}')?;
    // Look for "source":"value"
    for part in inner.split(',') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("\"source\"") {
            let rest = rest.trim().strip_prefix(':')?;
            let rest = rest.trim().trim_matches('"');
            if !rest.is_empty() {
                return Some(rest.to_string());
            }
        }
    }
    None
}

/// Parse an `ExternalFunction` from a JSON-like pragma value.
///
/// Expected format: `{"source":"module-name","importSpecifierName":"fnName"}`
///
/// Returns `None` if parsing fails. Both `source` and `importSpecifierName`
/// must be present.
fn parse_external_function_value(val: &str) -> Option<ExternalFunction> {
    let trimmed = val.trim();
    let inner = trimmed.strip_prefix('{')?.strip_suffix('}')?;
    let mut source = None;
    let mut import_specifier_name = None;
    for part in inner.split(',') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("\"source\"") {
            let rest = rest.trim().strip_prefix(':')?;
            let rest = rest.trim().trim_matches('"');
            if !rest.is_empty() {
                source = Some(rest.to_string());
            }
        } else if let Some(rest) = part.strip_prefix("\"importSpecifierName\"") {
            let rest = rest.trim().strip_prefix(':')?;
            let rest = rest.trim().trim_matches('"');
            if !rest.is_empty() {
                import_specifier_name = Some(rest.to_string());
            }
        }
    }
    Some(ExternalFunction { source: source?, import_specifier_name: import_specifier_name? })
}

/// Parse an inline JSON object for the `inlineJsxTransform` pragma value:
///
/// ```text
/// {"elementSymbol":"react.transitional.element","globalDevVar":"DEV"}
/// ```
///
/// Returns `None` if parsing fails. Both `elementSymbol` and `globalDevVar`
/// must be present.
fn parse_inline_jsx_transform_value(val: &str) -> Option<InlineJsxTransformConfig> {
    let trimmed = val.trim();
    let inner = trimmed.strip_prefix('{')?.strip_suffix('}')?;
    let mut element_symbol = None;
    let mut global_dev_var = None;
    for part in inner.split(',') {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("\"elementSymbol\"") {
            let rest = rest.trim().strip_prefix(':')?;
            let rest = rest.trim().trim_matches('"');
            if !rest.is_empty() {
                element_symbol = Some(rest.to_string());
            }
        } else if let Some(rest) = part.strip_prefix("\"globalDevVar\"") {
            let rest = rest.trim().strip_prefix(':')?;
            let rest = rest.trim().trim_matches('"');
            if !rest.is_empty() {
                global_dev_var = Some(rest.to_string());
            }
        }
    }
    Some(InlineJsxTransformConfig {
        element_symbol: element_symbol?,
        global_dev_var: global_dev_var?,
    })
}

fn parse_bool_value(value: Option<&String>, default: bool) -> bool {
    match value {
        None => default,
        Some(v) => match v.as_str() {
            "true" => true,
            "false" => false,
            _ => default,
        },
    }
}

/// Parse a simple JSON string array like `["Foo","Bar"]` into a `Vec<String>`.
fn parse_string_array(value: &str) -> Vec<String> {
    let trimmed = value.trim();
    if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
        return Vec::new();
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .filter_map(|s| {
            let s = s.trim().trim_matches('"');
            if s.is_empty() { None } else { Some(s.to_string()) }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_pragma() {
        let options = parse_config_pragma_for_tests(
            "",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        assert_eq!(options.compilation_mode, CompilationMode::All);
        assert_eq!(options.panic_threshold, PanicThreshold::AllErrors);
    }

    #[test]
    fn test_parse_enable_flags() {
        let options = parse_config_pragma_for_tests(
            "@enableForest @validateNoSetStateInEffects:true @validateNoSetStateInRender:false",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        assert!(options.environment.enable_forest);
        assert!(options.environment.validate_no_set_state_in_effects);
        assert!(!options.environment.validate_no_set_state_in_render);
    }

    #[test]
    fn test_parse_compilation_mode() {
        let options = parse_config_pragma_for_tests(
            "@compilationMode:infer",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        assert_eq!(options.compilation_mode, CompilationMode::Infer);
    }

    /// Port of `parseConfigPragma-test.ts` — "parses flags in various forms"
    #[test]
    fn parses_flags_in_various_forms() {
        // Validate defaults first to make sure that the parser is getting the value
        // from the pragma, and not just missing it and getting the default value
        let default_config = EnvironmentConfig::default();
        assert!(!default_config.enable_forest);
        assert!(!default_config.validate_no_set_state_in_effects);
        assert!(default_config.validate_no_set_state_in_render);

        let options = parse_config_pragma_for_tests(
            "@enableForest @validateNoSetStateInEffects:true @validateNoSetStateInRender:false",
            &PragmaDefaults { compilation_mode: CompilationMode::Infer },
        );

        // panicThreshold is overridden to AllErrors by parse_config_pragma_for_tests
        assert_eq!(options.panic_threshold, PanicThreshold::AllErrors);
        // compilationMode comes from the defaults
        assert_eq!(options.compilation_mode, CompilationMode::Infer);

        // Environment flags parsed from pragma
        assert!(options.environment.enable_forest);
        assert!(options.environment.validate_no_set_state_in_effects);
        assert!(!options.environment.validate_no_set_state_in_render);

        // enableResetCacheOnSourceFileChanges defaults to Some(false)
        // in test utils to reduce visual noise
        assert_eq!(options.environment.enable_reset_cache_on_source_file_changes, Some(false));
    }

    /// Verify that enableResetCacheOnSourceFileChanges can be explicitly enabled via pragma.
    #[test]
    fn parses_enable_reset_cache_on_source_file_changes() {
        let options = parse_config_pragma_for_tests(
            "@enableResetCacheOnSourceFileChanges",
            &PragmaDefaults { compilation_mode: CompilationMode::Infer },
        );
        assert_eq!(options.environment.enable_reset_cache_on_source_file_changes, Some(true));
    }

    #[test]
    fn parses_target_react17() {
        let options = parse_config_pragma_for_tests(
            "@target:\"17\"",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        assert_eq!(options.target, CompilerReactTarget::React17);
    }

    #[test]
    fn parses_target_react18() {
        let options = parse_config_pragma_for_tests(
            "@target:\"18\"",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        assert_eq!(options.target, CompilerReactTarget::React18);
    }

    #[test]
    fn parses_target_react19() {
        let options = parse_config_pragma_for_tests(
            "@target:\"19\"",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        assert_eq!(options.target, CompilerReactTarget::React19);
    }

    #[test]
    fn parses_target_meta_internal() {
        let options = parse_config_pragma_for_tests(
            "@target:\"donotuse_meta_internal\"",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        assert_eq!(
            options.target,
            CompilerReactTarget::MetaInternal { runtime_module: "react".to_string() }
        );
    }

    #[test]
    fn parses_target_without_value_keeps_default() {
        let options = parse_config_pragma_for_tests(
            "@target",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        // No value provided — target stays at default (React19)
        assert_eq!(options.target, CompilerReactTarget::React19);
    }

    #[test]
    fn parses_gating_without_value_uses_defaults() {
        let options = parse_config_pragma_for_tests(
            "@gating",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        let gating = options.gating.expect("gating should be set");
        assert_eq!(gating.source, "ReactForgetFeatureFlag");
        assert_eq!(gating.import_specifier_name, "isForgetEnabled_Fixtures");
    }

    #[test]
    fn parses_gating_with_json_value() {
        let options = parse_config_pragma_for_tests(
            r#"@gating:{"source":"shared-runtime","importSpecifierName":"getTrue"}"#,
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        let gating = options.gating.expect("gating should be set");
        assert_eq!(gating.source, "shared-runtime");
        assert_eq!(gating.import_specifier_name, "getTrue");
    }

    #[test]
    fn parses_gating_with_other_pragmas() {
        let options = parse_config_pragma_for_tests(
            "@gating @panicThreshold:\"none\" @compilationMode:\"infer\"",
            &PragmaDefaults { compilation_mode: CompilationMode::All },
        );
        let gating = options.gating.expect("gating should be set");
        assert_eq!(gating.source, "ReactForgetFeatureFlag");
        assert_eq!(gating.import_specifier_name, "isForgetEnabled_Fixtures");
        assert_eq!(options.panic_threshold, PanicThreshold::None);
        assert_eq!(options.compilation_mode, CompilationMode::Infer);
    }
}
