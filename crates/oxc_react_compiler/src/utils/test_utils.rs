/// Test utilities for the React Compiler.
///
/// Port of `Utils/TestUtils.ts` from the React Compiler.
///
/// Provides utilities for parsing test configuration pragmas from
/// fixture files and setting up the compiler for test execution.
use crate::entrypoint::options::{CompilationMode, PanicThreshold, PluginOptions};
use crate::hir::environment::EnvironmentConfig;

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

    let mut env_config = EnvironmentConfig::default();

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
            "validatePreserveExistingMemoizationGuarantees" => {
                env_config.validate_preserve_existing_memoization_guarantees =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enablePreserveExistingMemoizationGuarantees" => {
                env_config.enable_preserve_existing_memoization_guarantees =
                    parse_bool_value(entry.value.as_ref(), true);
            }
            "enableResetCacheOnSourceFileChanges" => {
                env_config.enable_reset_cache_on_source_file_changes =
                    Some(parse_bool_value(entry.value.as_ref(), true));
            }
            "compilationMode" => {
                if let Some(val) = &entry.value {
                    options.compilation_mode = match val.as_str() {
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
                    options.panic_threshold = match val.as_str() {
                        "critical_errors" => PanicThreshold::CriticalErrors,
                        "none" => PanicThreshold::None,
                        _ => PanicThreshold::AllErrors,
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
}
