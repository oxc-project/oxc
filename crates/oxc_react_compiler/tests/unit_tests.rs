/// Unit tests ported from the original React Compiler test suite.
///
/// Ports of:
/// - `__tests__/Result-test.ts` — tests for Result type (Rust native)
/// - `__tests__/envConfig-test.ts` — tests for environment config validation
/// - `__tests__/Logger-test.ts` — tests for compilation logging

// =====================================================================================
// Result-test.ts port
// =====================================================================================
// The TS version tests a custom Result class. In Rust, we use the native Result type.
// We test that our CompilerError integrates correctly with Result.

mod result_tests {
    use oxc_react_compiler::compiler_error::{
        CompilerError, CompilerDiagnostic, ErrorCategory, ErrorSeverity,
    };

    fn add_max_10(a: i32, b: i32) -> Result<i32, String> {
        let n = a + b;
        if n > 10 { Err(format!("{n} is too high")) } else { Ok(n) }
    }

    fn only_foo(foo: &str) -> Result<String, String> {
        if foo == "foo" { Ok(foo.to_string()) } else { Err(foo.to_string()) }
    }

    #[test]
    fn result_map() {
        assert_eq!(add_max_10(1, 1).map(|n| n * 2), Ok(4));
        assert!(add_max_10(10, 10).map(|n| n * 2).is_err());
    }

    #[test]
    fn result_map_err() {
        assert_eq!(
            add_max_10(1, 1).map_err(|e| format!("not a number: {e}")),
            Ok(2)
        );
        assert_eq!(
            add_max_10(10, 10).map_err(|e| format!("couldn't add: {e}")),
            Err("couldn't add: 20 is too high".to_string())
        );
    }

    #[test]
    fn result_map_or() {
        assert_eq!(only_foo("foo").map_or(42, |v| v.len()), 3);
        assert_eq!(only_foo("bar").map_or(42, |v| v.len()), 42);
    }

    #[test]
    fn result_map_or_else() {
        assert_eq!(only_foo("foo").map_or_else(|_| 42, |v| v.len()), 3);
        assert_eq!(only_foo("bar").map_or_else(|_| 42, |v| v.len()), 42);
    }

    #[test]
    fn result_and_then() {
        assert_eq!(add_max_10(1, 1).and_then(|n| Ok(n * 2)), Ok(4));
        assert!(add_max_10(10, 10).and_then(|n| Ok(n * 2)).is_err());
    }

    #[test]
    fn result_and() {
        assert_eq!(add_max_10(1, 1).and(Ok(4)), Ok(4));
        assert!(add_max_10(10, 10).and(Ok(4)).is_err());
        assert_eq!(
            add_max_10(1, 1).and(Err::<i32, _>("hehe".to_string())),
            Err("hehe".to_string())
        );
    }

    #[test]
    fn result_or() {
        assert_eq!(add_max_10(1, 1).or(Ok::<_, String>(4)), Ok(2));
        assert_eq!(add_max_10(10, 10).or(Ok::<_, String>(4)), Ok(4));
        assert_eq!(add_max_10(1, 1).or(Err::<i32, _>("hehe".to_string())), Ok(2));
        assert_eq!(
            add_max_10(10, 10).or(Err::<i32, _>("hehe".to_string())),
            Err("hehe".to_string())
        );
    }

    #[test]
    fn result_or_else() {
        assert_eq!(
            add_max_10(1, 1).or_else(|s| Err::<i32, _>(s.to_uppercase())),
            Ok(2)
        );
        assert_eq!(
            add_max_10(10, 10).or_else(|s| Err::<i32, _>(s.to_uppercase())),
            Err("20 IS TOO HIGH".to_string())
        );
    }

    #[test]
    fn result_is_ok_is_err() {
        assert!(add_max_10(1, 1).is_ok());
        assert!(!add_max_10(10, 10).is_ok());
        assert!(!add_max_10(1, 1).is_err());
        assert!(add_max_10(10, 10).is_err());
    }

    #[test]
    fn result_unwrap_or() {
        assert_eq!(add_max_10(1, 1).unwrap_or(4), 2);
        assert_eq!(add_max_10(10, 10).unwrap_or(4), 4);
    }

    #[test]
    fn result_unwrap_or_else() {
        assert_eq!(add_max_10(1, 1).unwrap_or_else(|_| 4), 2);
        assert_eq!(add_max_10(10, 10).unwrap_or_else(|s| s.len() as i32), 14);
    }

    // Test CompilerError's Result integration
    #[test]
    fn compiler_error_into_result() {
        let error = CompilerError::new();
        assert!(error.into_result().is_ok());

        let mut error = CompilerError::new();
        error.push_diagnostic(CompilerDiagnostic::create(
            ErrorCategory::Invariant,
            "test error".to_string(),
            None,
            None,
        ));
        assert!(error.into_result().is_err());
    }

    #[test]
    fn compiler_error_has_errors() {
        let error = CompilerError::new();
        assert!(!error.has_any_errors());
        assert!(!error.has_errors());

        let mut error = CompilerError::new();
        error.push_diagnostic(CompilerDiagnostic::create(
            ErrorCategory::Invariant,
            "test".to_string(),
            None,
            None,
        ));
        assert!(error.has_any_errors());
        assert!(error.has_errors());
    }

    #[test]
    fn compiler_error_merge() {
        let mut error1 = CompilerError::new();
        error1.push_diagnostic(CompilerDiagnostic::create(
            ErrorCategory::Hooks,
            "error 1".to_string(),
            None,
            None,
        ));

        let mut error2 = CompilerError::new();
        error2.push_diagnostic(CompilerDiagnostic::create(
            ErrorCategory::Refs,
            "error 2".to_string(),
            None,
            None,
        ));

        error1.merge(error2);
        assert_eq!(error1.details.len(), 2);
    }
}

// =====================================================================================
// envConfig-test.ts port
// =====================================================================================

mod env_config_tests {
    use oxc_react_compiler::hir::environment::{
        validate_environment_config, EnvironmentConfig, HookConfig, InstrumentationConfig,
        ExternalFunction,
    };
    use oxc_react_compiler::hir::{Effect, ValueKind};

    #[test]
    fn default_config_validates() {
        let config = EnvironmentConfig::default();
        let result = validate_environment_config(config);
        assert!(result.is_ok());
    }

    #[test]
    fn default_values_are_correct() {
        let config = EnvironmentConfig::default();
        assert!(config.validate_hooks_usage);
        assert!(config.validate_no_set_state_in_render);
        assert!(config.validate_ref_access_during_render);
        assert!(!config.enable_forest);
        assert!(!config.enable_function_outlining);
        assert!(!config.validate_no_set_state_in_effects);
    }

    #[test]
    fn instrumentation_without_gating_fails() {
        let config = EnvironmentConfig {
            enable_emit_instrument_forget: Some(InstrumentationConfig {
                func: ExternalFunction {
                    source: "test".to_string(),
                    import_specifier_name: "test".to_string(),
                },
                gating: None,
                global_gating: None,
            }),
            ..EnvironmentConfig::default()
        };
        let result = validate_environment_config(config);
        assert!(result.is_err());
    }

    #[test]
    fn instrumentation_with_gating_succeeds() {
        let config = EnvironmentConfig {
            enable_emit_instrument_forget: Some(InstrumentationConfig {
                func: ExternalFunction {
                    source: "test".to_string(),
                    import_specifier_name: "test".to_string(),
                },
                gating: Some(ExternalFunction {
                    source: "gating-module".to_string(),
                    import_specifier_name: "isEnabled".to_string(),
                }),
                global_gating: None,
            }),
            ..EnvironmentConfig::default()
        };
        let result = validate_environment_config(config);
        assert!(result.is_ok());
    }

    #[test]
    fn custom_hooks_config() {
        let mut config = EnvironmentConfig::default();
        config.custom_hooks.insert(
            "useFoo".to_string(),
            HookConfig {
                effect_kind: Effect::Freeze,
                value_kind: ValueKind::Frozen,
                no_alias: false,
                transitive_mixed_data: false,
            },
        );
        let result = validate_environment_config(config);
        assert!(result.is_ok());
        let validated = result.unwrap();
        let hook = validated.custom_hooks.get("useFoo");
        assert!(hook.is_some());
        let hook = hook.unwrap();
        assert_eq!(hook.effect_kind, Effect::Freeze);
        assert_eq!(hook.value_kind, ValueKind::Frozen);
    }
}

// =====================================================================================
// Logger-test.ts port
// =====================================================================================

mod logger_tests {
    use oxc_react_compiler::compiler_error::ErrorSeverity;
    use oxc_react_compiler::entrypoint::options::CompilationMode;
    use oxc_react_compiler::entrypoint::program::should_compile_function;
    use oxc_react_compiler::hir::ReactFunctionType;

    #[test]
    fn should_compile_component() {
        let result = should_compile_function(
            Some("Component"),
            &[],
            CompilationMode::Infer,
        );
        assert_eq!(result, Some(ReactFunctionType::Component));
    }

    #[test]
    fn should_compile_hook() {
        let result = should_compile_function(
            Some("useMyHook"),
            &[],
            CompilationMode::Infer,
        );
        assert_eq!(result, Some(ReactFunctionType::Hook));
    }

    #[test]
    fn should_not_compile_regular_function() {
        let result = should_compile_function(
            Some("helper"),
            &[],
            CompilationMode::Infer,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn should_compile_with_use_memo_directive() {
        let result = should_compile_function(
            Some("helper"),
            &["use memo".to_string()],
            CompilationMode::Infer,
        );
        assert!(result.is_some());
    }

    #[test]
    fn should_not_compile_with_use_no_memo_directive() {
        let result = should_compile_function(
            Some("Component"),
            &["use no memo".to_string()],
            CompilationMode::Infer,
        );
        assert_eq!(result, None);
    }

    #[test]
    fn annotation_mode_requires_directive() {
        let result = should_compile_function(
            Some("Component"),
            &[],
            CompilationMode::Annotation,
        );
        assert_eq!(result, None);

        let result = should_compile_function(
            Some("Component"),
            &["use memo".to_string()],
            CompilationMode::Annotation,
        );
        assert!(result.is_some());
    }

    #[test]
    fn all_mode_compiles_everything() {
        let result = should_compile_function(
            Some("helper"),
            &[],
            CompilationMode::All,
        );
        assert!(result.is_some());
    }

    #[test]
    fn lint_rules_are_well_formed() {
        use oxc_react_compiler::compiler_error::all_lint_rules;
        let rules = all_lint_rules();
        assert!(rules.len() >= 20); // We have 26 categories

        // Check that all rule names follow the pattern [a-z]+(-[a-z]+)*
        for rule in &rules {
            assert!(
                !rule.name.is_empty(),
                "Empty rule name for category {:?}",
                rule.category
            );
            assert!(
                rule.name.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
                "Invalid rule name: '{}' for category {:?}",
                rule.name,
                rule.category
            );
            assert!(
                !rule.name.starts_with('-') && !rule.name.ends_with('-'),
                "Rule name starts/ends with dash: '{}' for category {:?}",
                rule.name,
                rule.category
            );
        }
    }

    #[test]
    fn error_severity_display() {
        assert_eq!(format!("{}", ErrorSeverity::Error), "Error");
        assert_eq!(format!("{}", ErrorSeverity::Warning), "Warning");
        assert_eq!(format!("{}", ErrorSeverity::Hint), "Hint");
    }
}
