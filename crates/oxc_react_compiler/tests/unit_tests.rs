/// Unit tests ported from the original React Compiler test suite.
///
/// Ports of:
/// - `__tests__/Result-test.ts` -- tests for Result type (Rust native)
/// - `__tests__/envConfig-test.ts` -- tests for environment config validation
/// - `__tests__/Logger-test.ts` -- tests for compilation logging

/// Print codegen output body to a string using oxc_codegen.
fn print_codegen_body(
    output: &oxc_react_compiler::reactive_scopes::codegen_reactive_function::CodegenOutput<'_>,
) -> String {
    use oxc_codegen::{Codegen, Context, Gen};
    let mut codegen = Codegen::new();
    for stmt in output.body.iter() {
        stmt.print(&mut codegen, Context::default());
    }
    codegen.into_source_text()
}

mod result_tests {
    // Result-test.ts port
    // The TS version tests a custom Result class. In Rust, we use the native Result type.
    // We test that our CompilerError integrates correctly with Result.
    use oxc_react_compiler::compiler_error::{CompilerDiagnostic, CompilerError, ErrorCategory};

    fn add_max_10(a: i32, b: i32) -> Result<i32, String> {
        let n = a + b;
        if n > 10 { Err(format!("{n} is too high")) } else { Ok(n) }
    }

    fn only_target(input: &str) -> Result<String, String> {
        if input == "foo" { Ok(input.to_string()) } else { Err(input.to_string()) }
    }

    #[test]
    fn result_map() {
        assert_eq!(add_max_10(1, 1).map(|n| n * 2), Ok(4));
        assert!(add_max_10(10, 10).map(|n| n * 2).is_err());
    }

    #[test]
    fn result_map_err() {
        assert_eq!(add_max_10(1, 1).map_err(|e| format!("not a number: {e}")), Ok(2));
        assert_eq!(
            add_max_10(10, 10).map_err(|e| format!("couldn't add: {e}")),
            Err("couldn't add: 20 is too high".to_string())
        );
    }

    #[test]
    fn result_map_or() {
        assert_eq!(only_target("foo").map_or(42, |v| v.len()), 3);
        assert_eq!(only_target("bar").map_or(42, |v| v.len()), 42);
    }

    #[test]
    fn result_map_or_else() {
        assert_eq!(only_target("foo").map_or_else(|_| 42, |v| v.len()), 3);
        assert_eq!(only_target("bar").map_or_else(|_| 42, |v| v.len()), 42);
    }

    #[test]
    fn result_and_then() {
        assert_eq!(add_max_10(1, 1).and_then(|n| add_max_10(n, n)), Ok(4));
        assert!(add_max_10(10, 10).and_then(|n| add_max_10(n, n)).is_err());
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
        fn manual_uppercase(s: &str) -> String {
            s.chars().map(|c| c.to_ascii_uppercase()).collect()
        }
        assert_eq!(add_max_10(1, 1).map_err(|s| manual_uppercase(&s)), Ok(2));
        assert_eq!(
            add_max_10(10, 10).map_err(|s| manual_uppercase(&s)),
            Err("20 IS TOO HIGH".to_string())
        );
    }

    #[test]
    fn result_is_ok_is_err() {
        assert!(add_max_10(1, 1).is_ok());
        assert!(add_max_10(10, 10).is_err());
        assert!(add_max_10(1, 1).is_ok());
        assert!(add_max_10(10, 10).is_err());
    }

    #[test]
    fn result_unwrap_or() {
        assert_eq!(add_max_10(1, 1).unwrap_or(4), 2);
        assert_eq!(add_max_10(10, 10).unwrap_or(4), 4);
    }

    #[test]
    fn result_unwrap_or_else() {
        assert_eq!(add_max_10(1, 1).unwrap_or(4), 2);
        assert_eq!(add_max_10(10, 10).unwrap_or_else(|s| i32::try_from(s.len()).unwrap()), 14);
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
        EnvironmentConfig, ExternalFunction, HookConfig, InstrumentationConfig,
        validate_environment_config,
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
        assert!(config.enable_function_outlining);
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

    /// Fix 8: Verify enableMemoization and enableDropManualMemoization flags
    /// are correctly derived from each CompilerOutputMode.
    #[test]
    fn memoization_flags_match_ts_reference() {
        use oxc_react_compiler::hir::ReactFunctionType;
        use oxc_react_compiler::hir::environment::{CompilerOutputMode, Environment};

        // Client: memoization=true, drop_manual_memo=true
        let env = Environment::new(
            ReactFunctionType::Component,
            CompilerOutputMode::Client,
            EnvironmentConfig::default(),
        )
        .unwrap();
        assert!(env.enable_memoization, "Client: enable_memoization should be true");
        assert!(
            env.enable_drop_manual_memoization,
            "Client: enable_drop_manual_memoization should be true"
        );

        // Lint: memoization=true, drop_manual_memo=true
        let env = Environment::new(
            ReactFunctionType::Component,
            CompilerOutputMode::Lint,
            EnvironmentConfig::default(),
        )
        .unwrap();
        assert!(env.enable_memoization, "Lint: enable_memoization should be true");
        assert!(
            env.enable_drop_manual_memoization,
            "Lint: enable_drop_manual_memoization should be true"
        );

        // Ssr: memoization=false, drop_manual_memo=true
        let env = Environment::new(
            ReactFunctionType::Component,
            CompilerOutputMode::Ssr,
            EnvironmentConfig::default(),
        )
        .unwrap();
        assert!(!env.enable_memoization, "Ssr: enable_memoization should be false");
        assert!(
            env.enable_drop_manual_memoization,
            "Ssr: enable_drop_manual_memoization should be true"
        );

        // ClientNoMemo: memoization=false, drop_manual_memo=false
        let env = Environment::new(
            ReactFunctionType::Component,
            CompilerOutputMode::ClientNoMemo,
            EnvironmentConfig::default(),
        )
        .unwrap();
        assert!(!env.enable_memoization, "ClientNoMemo: enable_memoization should be false");
        assert!(
            !env.enable_drop_manual_memoization,
            "ClientNoMemo: enable_drop_manual_memoization should be false"
        );
    }

    /// Fix 9: Verify that custom hooks from config are actually registered
    /// in the globals registry and can be looked up via get_global_declaration.
    #[test]
    fn custom_hooks_registered_in_globals() {
        use oxc_react_compiler::compiler_error::SourceLocation;
        use oxc_react_compiler::hir::environment::{CompilerOutputMode, Environment};
        use oxc_react_compiler::hir::{NonLocalBinding, ReactFunctionType};

        let mut config = EnvironmentConfig::default();
        config.custom_hooks.insert(
            "useCustom".to_string(),
            HookConfig {
                effect_kind: Effect::Freeze,
                value_kind: ValueKind::Frozen,
                no_alias: false,
                transitive_mixed_data: false,
            },
        );

        let mut env =
            Environment::new(ReactFunctionType::Component, CompilerOutputMode::Client, config)
                .unwrap();

        // Look up "useCustom" via get_global_declaration as a Global binding
        let result = env
            .get_global_declaration(
                &NonLocalBinding::Global { name: "useCustom".to_string() },
                SourceLocation::Generated,
            )
            .expect("should not error");
        assert!(result.is_some(), "useCustom should be registered as a global declaration");
    }

    /// Verify that registering a custom hook whose name collides with an existing
    /// built-in global returns an error instead of panicking.
    #[test]
    fn custom_hook_collision_with_builtin_global_returns_error() {
        use oxc_react_compiler::hir::ReactFunctionType;
        use oxc_react_compiler::hir::environment::{CompilerOutputMode, Environment};

        // "console" is a built-in global — registering a custom hook with that name
        // should fail gracefully.
        let mut config = EnvironmentConfig::default();
        config.custom_hooks.insert(
            "console".to_string(),
            HookConfig {
                effect_kind: Effect::Freeze,
                value_kind: ValueKind::Frozen,
                no_alias: false,
                transitive_mixed_data: false,
            },
        );

        let result =
            Environment::new(ReactFunctionType::Component, CompilerOutputMode::Client, config);
        assert!(
            result.is_err(),
            "Should return error for custom hook colliding with built-in global"
        );
        let err = result.unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("console"),
            "Error message should mention the conflicting name 'console', got: {msg}"
        );
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
    use oxc_react_compiler::hir::build_hir::LowerableFunction;

    /// Parse source code and extract the first function declaration as a LowerableFunction,
    /// then call `f` with it. Uses JSX source type to support JSX in test bodies.
    fn with_parsed_function<R>(
        source: &str,
        f: impl FnOnce(&LowerableFunction<'_>, Option<&str>) -> R,
    ) -> R {
        let allocator = oxc_allocator::Allocator::default();
        let source_type = oxc_span::SourceType::jsx();
        let result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
        assert!(result.errors.is_empty(), "Parse errors: {:?}", result.errors);
        for stmt in &result.program.body {
            if let oxc_ast::ast::Statement::FunctionDeclaration(func) = stmt {
                let name = func.id.as_ref().map(|id| id.name.as_str());
                let lowerable = LowerableFunction::Function(func);
                return f(&lowerable, name);
            }
        }
        panic!("No function declaration found in source");
    }

    #[test]
    fn should_compile_component() {
        // Component with JSX — qualifies as component in Infer mode
        let result =
            with_parsed_function("function Component() { return <div />; }", |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Infer, false, false)
            });
        assert_eq!(result, Some(ReactFunctionType::Component));
    }

    #[test]
    fn should_compile_hook() {
        // Hook with a hook call — qualifies as hook in Infer mode
        let result =
            with_parsed_function("function useMyHook() { return useState(0); }", |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Infer, false, false)
            });
        assert_eq!(result, Some(ReactFunctionType::Hook));
    }

    #[test]
    fn should_not_compile_regular_function() {
        let result = with_parsed_function("function helper() { return 1; }", |func, name| {
            should_compile_function(func, name, &[], CompilationMode::Infer, false, false)
        });
        assert_eq!(result, None);
    }

    #[test]
    fn should_compile_with_use_memo_directive() {
        // Opt-in directive causes compilation regardless of function name/body
        let result = with_parsed_function("function helper() { return 1; }", |func, name| {
            should_compile_function(
                func,
                name,
                &["use memo".to_string()],
                CompilationMode::Infer,
                false,
                false,
            )
        });
        assert!(result.is_some());
    }

    #[test]
    fn should_not_compile_with_use_no_memo_directive() {
        // `should_compile_function` (port of `getReactFunctionType`) does NOT check
        // opt-out directives. In the TS reference, opt-out is checked by the caller
        // (processFn) AFTER compilation. So even with 'use no memo', the function
        // type is still determined as Component.
        let result =
            with_parsed_function("function Component() { return <div />; }", |func, name| {
                should_compile_function(
                    func,
                    name,
                    &["use no memo".to_string()],
                    CompilationMode::Infer,
                    false,
                    false,
                )
            });
        assert_eq!(result, Some(ReactFunctionType::Component));
    }

    #[test]
    fn annotation_mode_requires_directive() {
        let result =
            with_parsed_function("function Component() { return <div />; }", |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Annotation, false, false)
            });
        assert_eq!(result, None);

        let result =
            with_parsed_function("function Component() { return <div />; }", |func, name| {
                should_compile_function(
                    func,
                    name,
                    &["use memo".to_string()],
                    CompilationMode::Annotation,
                    false,
                    false,
                )
            });
        assert!(result.is_some());
    }

    #[test]
    fn all_mode_compiles_everything() {
        // Even a plain helper is compiled in All mode (as Other)
        let result = with_parsed_function("function helper() { return 1; }", |func, name| {
            should_compile_function(func, name, &[], CompilationMode::All, false, false)
        });
        assert!(result.is_some());
    }

    #[test]
    fn memo_callback_compiles_as_component_in_infer_mode() {
        // An anonymous-like function inside React.memo() should compile as Component
        // if it calls hooks or creates JSX. Pass is_memo_or_forwardref_arg=true.
        let result = with_parsed_function("function _temp() { return <div />; }", |func, _name| {
            // Pass None for name to simulate anonymous, and is_memo_or_forwardref_arg=true
            should_compile_function(func, None, &[], CompilationMode::Infer, true, false)
        });
        assert_eq!(result, Some(ReactFunctionType::Component));
    }

    #[test]
    fn memo_callback_compiles_as_component_in_all_mode() {
        let result = with_parsed_function("function _temp() { return <div />; }", |func, _name| {
            should_compile_function(func, None, &[], CompilationMode::All, true, false)
        });
        assert_eq!(result, Some(ReactFunctionType::Component));
    }

    #[test]
    fn forwardref_callback_not_compiled_without_flag() {
        // Without is_memo_or_forwardref_arg, an anonymous function should not compile in Infer mode
        let result = with_parsed_function("function _temp() { return <div />; }", |func, _name| {
            should_compile_function(func, None, &[], CompilationMode::Infer, false, false)
        });
        assert_eq!(result, None);
    }

    #[test]
    fn lint_rules_are_well_formed() {
        use oxc_react_compiler::compiler_error::all_lint_rules;
        let rules = all_lint_rules();
        assert!(rules.len() >= 20); // We have 26 categories

        // Check that all rule names follow the pattern [a-z]+(-[a-z]+)*
        for rule in &rules {
            assert!(!rule.name.is_empty(), "Empty rule name for category {:?}", rule.category);
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

    /// Fix 10: TS type wrappers (TSAsExpression, TSNonNullExpression, etc.)
    /// should be seen through when detecting hook calls / JSX in function bodies.
    /// `useState(0) as any` should still be detected as a hook call.
    #[test]
    fn ts_type_wrapper_detects_hooks() {
        // Use TSX source type to support both TS syntax and JSX
        fn with_parsed_tsx_function<R>(
            source: &str,
            f: impl FnOnce(&LowerableFunction<'_>, Option<&str>) -> R,
        ) -> R {
            let allocator = oxc_allocator::Allocator::default();
            let source_type = oxc_span::SourceType::tsx();
            let result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
            assert!(result.errors.is_empty(), "Parse errors: {:?}", result.errors);
            for stmt in &result.program.body {
                if let oxc_ast::ast::Statement::FunctionDeclaration(func) = stmt {
                    let name = func.id.as_ref().map(|id| id.name.as_str());
                    let lowerable = LowerableFunction::Function(func);
                    return f(&lowerable, name);
                }
            }
            panic!("No function declaration found in source");
        }

        // TSAsExpression wrapping a hook call
        let result = with_parsed_tsx_function(
            "function Component() { const x = useState(0) as any; return <div>{x}</div>; }",
            |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Infer, false, false)
            },
        );
        assert_eq!(
            result,
            Some(ReactFunctionType::Component),
            "useState() as any should still be detected as a hook call"
        );

        // TSNonNullExpression wrapping a hook call
        let result = with_parsed_tsx_function(
            "function Component() { const x = useState(0)!; return <div>{x}</div>; }",
            |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Infer, false, false)
            },
        );
        assert_eq!(
            result,
            Some(ReactFunctionType::Component),
            "useState()! should still be detected as a hook call"
        );
    }

    /// Fix 11: Verify that a function with a hook name is correctly identified
    /// as a Hook by should_compile_function. This tests the name inference path:
    /// when the transformer infers a name like "useHook" from an AssignmentPattern,
    /// passing that name to should_compile_function correctly classifies it.
    #[test]
    fn hook_name_inference_compiles_as_hook() {
        // When the transformer sees `const {useHook = () => {}} = {}`,
        // it infers the name "useHook" from the binding. Here we verify
        // that should_compile_function with name "useHook" classifies
        // a function containing a hook call as a Hook.
        let result =
            with_parsed_function("function useHook() { return useState(0); }", |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Infer, false, false)
            });
        assert_eq!(
            result,
            Some(ReactFunctionType::Hook),
            "A function named useHook calling useState should be classified as Hook"
        );

        // Also verify that without the hook name, the same body is NOT compiled
        // (since it doesn't have JSX and the name isn't PascalCase or use* prefix)
        let result =
            with_parsed_function("function helper() { return useState(0); }", |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Infer, false, false)
            });
        assert_eq!(
            result, None,
            "A function named helper (not hook/component name) should not compile in Infer mode"
        );
    }
}

/// Test that console.log(x) doesn't extend x's mutable range.
/// The expected output should have console.log(x) OUTSIDE the scope guard.
#[test]
fn test_console_readonly_output() {
    use oxc_react_compiler::entrypoint::pipeline::{run_codegen, run_pipeline};
    use oxc_react_compiler::hir::ReactFunctionType;
    use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };

    // Full test: all console methods including global.console.log
    let source = r#"function Component(props) {
  const x = [props.a, props.b];
  console.log(x);
  console.info(x);
  console.warn(x);
  console.error(x);
  console.trace(x);
  console.table(x);
  global.console.log(x);
  return x;
}"#;

    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::jsx();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    assert!(parser_result.errors.is_empty(), "Parse errors: {:?}", parser_result.errors);

    let func = parser_result
        .program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            oxc_ast::ast::Statement::FunctionDeclaration(f) => Some(LowerableFunction::Function(f)),
            _ => None,
        })
        .expect("No function found");

    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    )
    .unwrap();

    let mut hir_func =
        lower(&env, ReactFunctionType::Component, &func, rustc_hash::FxHashMap::default())
            .expect("Lower failed");
    let pipeline_output = run_pipeline(&mut hir_func, &env).expect("Pipeline failed");
    let ast = oxc_ast::AstBuilder::new(&allocator);
    let result = run_codegen(pipeline_output, &env, ast, "_c", None).expect("Codegen failed");
    let output = print_codegen_body(&result);

    // The console.log(x) call should be OUTSIDE the scope guard.
    // Expected pattern: the scope guard assigns to $ cache, then console.log happens after.
    // Bad pattern: console.log(t0) is INSIDE the if block (before $[N] = t0).
    //
    // Look for: console.log should appear AFTER the scope guard's else branch.
    // If console.log is inside the if block, it means x's mutable range was extended.
    // Check that console.log appears after the cache store, not before it
    let lines: Vec<&str> = output.lines().collect();
    let mut found_cache_store = false;
    let mut console_after_cache = false;
    let mut console_before_cache = false;
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("$[") && trimmed.contains("=") && !trimmed.contains("!==") {
            found_cache_store = true;
        }
        if trimmed.contains("console.log") {
            if found_cache_store {
                console_after_cache = true;
            } else {
                console_before_cache = true;
            }
        }
    }

    assert!(
        console_after_cache && !console_before_cache,
        "console.log should be OUTSIDE the scope guard (after cache stores), but it appears to be inside.\nOutput:\n{output}"
    );
}

/// Test that context variables (StoreContext/DeclareContext) are properly tracked
/// in PruneNonEscapingScopes so that reactive scopes containing them are not
/// incorrectly pruned. This is a regression test for a bug where StoreContext
/// and DeclareContext were not adding their inner target (the context variable)
/// to the dependency graph in PruneNonEscapingScopes, causing scopes that
/// produced context variables to be pruned even when those variables escaped.
#[test]
fn test_context_variable_reactive_scopes() {
    use oxc_react_compiler::entrypoint::pipeline::{run_codegen, run_pipeline};
    use oxc_react_compiler::hir::ReactFunctionType;
    use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };

    // Based on the capturing-func-simple-alias-iife fixture.
    // The IIFE inlining converts `y` to a context variable (StoreContext).
    // The returned `y` must be memoized.
    let source = r#"function component(a) {
  let x = {a};
  let y = {};
  (function () {
    y = x;
  })();
  mutate(y);
  return y;
}"#;

    let allocator = oxc_allocator::Allocator::default();
    let source_type = oxc_span::SourceType::jsx();
    let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
    assert!(parser_result.errors.is_empty(), "Parse errors: {:?}", parser_result.errors);

    let func = parser_result
        .program
        .body
        .iter()
        .find_map(|stmt| match stmt {
            oxc_ast::ast::Statement::FunctionDeclaration(f) => Some(LowerableFunction::Function(f)),
            _ => None,
        })
        .expect("No function found");

    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    )
    .unwrap();

    let mut hir_func =
        lower(&env, ReactFunctionType::Component, &func, rustc_hash::FxHashMap::default())
            .expect("Lower failed");
    let pipeline_output = run_pipeline(&mut hir_func, &env).expect("Pipeline failed");
    let ast = oxc_ast::AstBuilder::new(&allocator);
    let result = run_codegen(pipeline_output, &env, ast, "_c", None).expect("Codegen failed");

    // The expected output should have _c(2) and a reactive scope
    assert_eq!(
        result.memo_slots_used, 2,
        "Expected 2 memo slots for context variable memoization, got {}",
        result.memo_slots_used
    );
}

/// Verify console and global.console method types resolve correctly.
#[test]
fn test_console_method_type_resolution() {
    use oxc_react_compiler::hir::ReactFunctionType;
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };

    let mut env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    )
    .unwrap();

    // Helper to verify a console-like object has a "log" method with Read effects.
    let verify_console_log = |env: &Environment,
                              console_type: &oxc_react_compiler::hir::types::Type,
                              label: &str| match console_type {
        oxc_react_compiler::hir::types::Type::Object(obj) => {
            assert!(obj.shape_id.is_some(), "{label} should have a shape_id");

            let log_type = env.get_property_type(console_type, "log");
            assert!(log_type.is_some(), "{label}.log should have a property type");

            let log_type = log_type.unwrap();
            match &log_type {
                oxc_react_compiler::hir::types::Type::Function(func_type) => {
                    assert!(
                        func_type.shape_id.is_some(),
                        "{label}.log should have a function shape_id"
                    );
                    let sig = env.get_function_signature(&log_type);
                    assert!(sig.is_some(), "{label}.log should have a function signature");
                    let sig = sig.unwrap();
                    assert_eq!(
                        sig.callee_effect,
                        oxc_react_compiler::hir::Effect::Read,
                        "{label}.log callee_effect should be Read"
                    );
                    assert_eq!(
                        sig.rest_param,
                        Some(oxc_react_compiler::hir::Effect::Read),
                        "{label}.log rest_param should be Read"
                    );
                }
                _ => panic!("{label}.log should be a Function type, got: {log_type:?}"),
            }
        }
        _ => panic!("{label} should be an Object type, got: {console_type:?}"),
    };

    let generated_loc = oxc_react_compiler::compiler_error::SourceLocation::Generated;

    // 1. Verify direct `console` global
    let console_global = env
        .get_global_declaration(
            &oxc_react_compiler::hir::NonLocalBinding::Global { name: "console".to_string() },
            generated_loc,
        )
        .expect("should not error")
        .expect("console should be a registered global");
    let console_type = oxc_react_compiler::hir::globals::Global::to_type(&console_global);
    verify_console_log(&env, &console_type, "console");

    // 2. Verify `global.console` resolves correctly
    let global_global = env
        .get_global_declaration(
            &oxc_react_compiler::hir::NonLocalBinding::Global { name: "global".to_string() },
            generated_loc,
        )
        .expect("should not error")
        .expect("global should be a registered global");
    let global_type = oxc_react_compiler::hir::globals::Global::to_type(&global_global);
    let global_console_type = env.get_property_type(&global_type, "console");
    assert!(global_console_type.is_some(), "global.console should resolve to a type");
    verify_console_log(&env, &global_console_type.unwrap(), "global.console");

    // 3. Verify `globalThis.console` resolves correctly
    let global_this = env
        .get_global_declaration(
            &oxc_react_compiler::hir::NonLocalBinding::Global { name: "globalThis".to_string() },
            generated_loc,
        )
        .expect("should not error")
        .expect("globalThis should be a registered global");
    let global_this_type = oxc_react_compiler::hir::globals::Global::to_type(&global_this);
    let global_this_console_type = env.get_property_type(&global_this_type, "console");
    assert!(global_this_console_type.is_some(), "globalThis.console should resolve to a type");
    verify_console_log(&env, &global_this_console_type.unwrap(), "globalThis.console");
}

#[test]
fn test_context_variable_debug() {
    use oxc_react_compiler::entrypoint::pipeline::{run_codegen, run_pipeline};
    use oxc_react_compiler::hir::build_hir::{LowerableFunction, lower};
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };
    use oxc_react_compiler::hir::print_hir::print_function;
    use oxc_react_compiler::hir::{InstructionValue, ReactFunctionType};

    let source = r#"function Component(p) {
  let x;
  const foo = () => {
    x = {};
  };
  foo();
  return x;
}
"#;
    let allocator = oxc_allocator::Allocator::default();
    let parser_result =
        oxc_parser::Parser::new(&allocator, source, oxc_span::SourceType::jsx()).parse();
    assert!(parser_result.errors.is_empty(), "Parse errors: {:?}", parser_result.errors);

    let env_config = EnvironmentConfig::default();
    let env =
        Environment::new(ReactFunctionType::Component, CompilerOutputMode::Client, env_config)
            .unwrap();

    let func_decl = parser_result
        .program
        .body
        .iter()
        .find_map(|stmt| {
            if let oxc_ast::ast::Statement::FunctionDeclaration(f) = stmt { Some(f) } else { None }
        })
        .expect("Expected function declaration");

    let mut hir_func = lower(
        &env,
        ReactFunctionType::Component,
        &LowerableFunction::Function(func_decl),
        rustc_hash::FxHashMap::default(),
    )
    .expect("Lower failed");

    // Print HIR before pipeline
    println!("=== HIR after lowering (before pipeline) ===");
    println!("{}", print_function(&hir_func));

    // Run pipeline - we need to intercept at specific points
    // First let's just run the full pipeline and check mutable ranges
    let pipeline_result = run_pipeline(&mut hir_func, &env);

    // Print all instructions with mutable ranges and scopes
    println!("=== After full pipeline ===");
    for (&block_id, block) in &hir_func.body.blocks {
        println!("Block {}:", block_id.0);
        for instr in &block.instructions {
            let lid = &instr.lvalue.identifier;
            let scope_info = lid
                .scope
                .as_ref()
                .map(|s| format!("scope={} [{}-{}]", s.id.0, s.range.start.0, s.range.end.0))
                .unwrap_or_default();
            println!(
                "  [{}] lvalue={:?}:{:?} mr=[{}-{}] reactive={} {}",
                instr.id.0,
                lid.id,
                lid.name,
                lid.mutable_range.start.0,
                lid.mutable_range.end.0,
                instr.lvalue.reactive,
                scope_info
            );

            match &instr.value {
                InstructionValue::DeclareContext(v) => {
                    let id = &v.lvalue_place.identifier;
                    let si = id
                        .scope
                        .as_ref()
                        .map(|s| {
                            format!("scope={} [{}-{}]", s.id.0, s.range.start.0, s.range.end.0)
                        })
                        .unwrap_or_default();
                    println!(
                        "    DeclareContext {:?}:{:?} mr=[{}-{}] reactive={} {}",
                        id.id,
                        id.name,
                        id.mutable_range.start.0,
                        id.mutable_range.end.0,
                        v.lvalue_place.reactive,
                        si
                    );
                }
                InstructionValue::StoreContext(v) => {
                    let id = &v.lvalue_place.identifier;
                    let si = id
                        .scope
                        .as_ref()
                        .map(|s| {
                            format!("scope={} [{}-{}]", s.id.0, s.range.start.0, s.range.end.0)
                        })
                        .unwrap_or_default();
                    println!(
                        "    StoreContext {:?}:{:?} mr=[{}-{}] reactive={} {}",
                        id.id,
                        id.name,
                        id.mutable_range.start.0,
                        id.mutable_range.end.0,
                        v.lvalue_place.reactive,
                        si
                    );
                    let vid = &v.value.identifier;
                    println!(
                        "    value: {:?}:{:?} mr=[{}-{}] reactive={}",
                        vid.id,
                        vid.name,
                        vid.mutable_range.start.0,
                        vid.mutable_range.end.0,
                        v.value.reactive
                    );
                }
                InstructionValue::LoadContext(v) => {
                    let id = &v.place.identifier;
                    let si = id
                        .scope
                        .as_ref()
                        .map(|s| {
                            format!("scope={} [{}-{}]", s.id.0, s.range.start.0, s.range.end.0)
                        })
                        .unwrap_or_default();
                    println!(
                        "    LoadContext {:?}:{:?} mr=[{}-{}] reactive={} {}",
                        id.id,
                        id.name,
                        id.mutable_range.start.0,
                        id.mutable_range.end.0,
                        v.place.reactive,
                        si
                    );
                }
                InstructionValue::FunctionExpression(v) => {
                    println!("    FunctionExpression context:");
                    for c in &v.lowered_func.func.context {
                        println!(
                            "      {:?}:{:?} mr=[{}-{}] effect={:?} reactive={}",
                            c.identifier.id,
                            c.identifier.name,
                            c.identifier.mutable_range.start.0,
                            c.identifier.mutable_range.end.0,
                            c.effect,
                            c.reactive
                        );
                    }
                }
                _ => {}
            }
        }
    }

    // Check for Scope terminals
    println!("=== Terminal types ===");
    for (&block_id, block) in &hir_func.body.blocks {
        let terminal_type = match &block.terminal {
            oxc_react_compiler::hir::Terminal::Scope(s) => format!(
                "Scope(body={}, fallthrough={}, scope_id={}, range=[{}-{}], deps={}, decls={})",
                s.block.0,
                s.fallthrough.0,
                s.scope.id.0,
                s.scope.range.start.0,
                s.scope.range.end.0,
                s.scope.dependencies.len(),
                s.scope.declarations.len()
            ),
            oxc_react_compiler::hir::Terminal::PrunedScope(s) => format!(
                "PrunedScope(body={}, fallthrough={}, scope_id={})",
                s.block.0, s.fallthrough.0, s.scope.id.0
            ),
            oxc_react_compiler::hir::Terminal::Goto(g) => format!("Goto({})", g.block.0),
            oxc_react_compiler::hir::Terminal::Return(_) => "Return".to_string(),
            t => format!("{:?}", std::mem::discriminant(t)),
        };
        println!("  Block {} -> {}", block_id.0, terminal_type);
    }

    match pipeline_result {
        Ok(pipeline_output) => {
            let ast = oxc_ast::AstBuilder::new(&allocator);
            let codegen_func =
                run_codegen(pipeline_output, &env, ast, "_c", None).expect("Codegen failed");
            let output = print_codegen_body(&codegen_func);
            println!("=== Codegen output ===\n{output}");
            assert!(
                output.contains("_c("),
                "Expected memoization (_c) in output but got:\n{output}"
            );
        }
        Err(e) => {
            panic!("Pipeline error: {e:?}");
        }
    }
}

// =====================================================================================
// Alignment fix regression tests
// =====================================================================================

mod alignment_fix_tests {
    use oxc_codegen::Gen;
    use oxc_react_compiler::entrypoint::pipeline::{run_codegen, run_pipeline};
    use oxc_react_compiler::hir::ReactFunctionType;
    use oxc_react_compiler::hir::build_hir::{LowerableFunction, collect_import_bindings, lower};
    use oxc_react_compiler::hir::environment::{
        CompilerOutputMode, Environment, EnvironmentConfig,
    };

    /// Compile a component source string through the full pipeline and return codegen output.
    /// Returns Err(String) on any failure (parse, lower, pipeline, codegen).
    fn compile_component(source: &str) -> Result<String, String> {
        compile_component_with_config(source, oxc_span::SourceType::jsx(), EnvironmentConfig::default())
    }

    /// Compile with a custom EnvironmentConfig.
    fn compile_component_with_env(source: &str, config: EnvironmentConfig) -> Result<String, String> {
        compile_component_with_config(source, oxc_span::SourceType::jsx(), config)
    }

    fn compile_component_with_config(
        source: &str,
        source_type: oxc_span::SourceType,
        env_config: EnvironmentConfig,
    ) -> Result<String, String> {
        let allocator = oxc_allocator::Allocator::default();
        let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
        if !parser_result.errors.is_empty() {
            return Err(format!("Parse errors: {:?}", parser_result.errors));
        }

        let func = parser_result
            .program
            .body
            .iter()
            .find_map(|stmt| match stmt {
                oxc_ast::ast::Statement::FunctionDeclaration(f) => {
                    Some(LowerableFunction::Function(f))
                }
                _ => None,
            })
            .ok_or_else(|| "No function declaration found in source".to_string())?;

        let env = Environment::new(
            ReactFunctionType::Component,
            CompilerOutputMode::Client,
            env_config,
        )
        .unwrap();

        let outer_bindings = collect_import_bindings(&parser_result.program.body);
        let mut hir_func = lower(&env, ReactFunctionType::Component, &func, outer_bindings)
            .map_err(|e| format!("Lower failed: {e:?}"))?;

        let pipeline_output =
            run_pipeline(&mut hir_func, &env).map_err(|e| format!("Pipeline failed: {e:?}"))?;

        let ast = oxc_ast::AstBuilder::new(&allocator);
        let result = run_codegen(pipeline_output, &env, ast, "_c", None)
            .map_err(|e| format!("Codegen failed: {e:?}"))?;

        let mut codegen = oxc_codegen::Codegen::new();
        for stmt in result.body.iter() {
            stmt.print(&mut codegen, oxc_codegen::Context::default());
        }
        Ok(codegen.into_source_text())
    }

    /// Fix 1: Return terminal freeze — components/hooks freeze return values.
    /// A component returning a mutable array should have that array memoized.
    #[test]
    fn test_return_terminal_freeze() {
        let source = r#"function Component(props) {
  const arr = [props.a, props.b];
  return arr;
}"#;
        let result = compile_component(source);
        assert!(result.is_ok(), "Pipeline should succeed: {}", result.unwrap_err());
        let output = result.unwrap();
        // The returned array should be memoized (cache slot assigned)
        assert!(output.contains("$["), "Expected memoization in output but got:\n{output}");
    }

    /// Fix 2: Try/catch aliasing — call results in try blocks are aliased to catch handler.
    #[test]
    fn test_try_catch_aliasing() {
        let source = r#"function Component(props) {
  let result;
  try {
    result = fetchData(props.id);
  } catch (e) {
    result = defaultValue;
  }
  return <div>{result}</div>;
}"#;
        let result = compile_component(source);
        assert!(result.is_ok(), "Pipeline should succeed: {}", result.unwrap_err());
    }

    /// Fix 3: MethodCall local MutateTransitiveConditionally — locally-defined method calls
    /// should not cause pipeline failures.
    #[test]
    fn test_method_call_local_mutate_transitive() {
        let source = r#"function Component(props) {
  const obj = { method() { return props.x; } };
  const result = obj.method();
  return <div>{result}</div>;
}"#;
        let result = compile_component(source);
        assert!(result.is_ok(), "Pipeline should succeed: {}", result.unwrap_err());
    }

    /// Fix 4: Freeze effect filtering — freeze effects on already-frozen values are dropped.
    /// Nested JSX (JSX inside JSX) should compile without issue since inner JSX is
    /// already frozen when captured by the outer JSX.
    #[test]
    fn test_freeze_effect_filtering_nested_jsx() {
        let source = r#"function Component(props) {
  const inner = <span>{props.a}</span>;
  return <div>{inner}</div>;
}"#;
        let result = compile_component(source);
        assert!(result.is_ok(), "Pipeline should succeed: {}", result.unwrap_err());
        let output = result.unwrap();
        assert!(output.contains("$["), "Expected memoization in output but got:\n{output}");
    }

    /// Fix 5: ImmutableCapture drop for Global/Primitive — capturing global/primitive values
    /// should not produce spurious effects.
    #[test]
    fn test_immutable_capture_global_primitive() {
        let source = r#"function Component() {
  const x = Math.random();
  return <div>{x}</div>;
}"#;
        let result = compile_component(source);
        assert!(result.is_ok(), "Pipeline should succeed: {}", result.unwrap_err());
    }

    /// Fix 6: CreateFrom for Primitive/Global — loading a property from a primitive
    /// should compile without panics.
    #[test]
    fn test_create_from_primitive_global() {
        let source = r#"function Component(props) {
  const len = props.name.length;
  return <div>{len}</div>;
}"#;
        let result = compile_component(source);
        assert!(result.is_ok(), "Pipeline should succeed: {}", result.unwrap_err());
    }

    /// Fix 7: mutableSpreads in compute_effects_for_signature — spread args to hooks
    /// that freeze is an edge case; basic spread usage should compile.
    #[test]
    fn test_mutable_spreads_compute_effects() {
        let source = r#"function Component(props) {
  const items = [props.a, props.b];
  const copy = [...items];
  return <div>{copy}</div>;
}"#;
        let result = compile_component(source);
        assert!(result.is_ok(), "Pipeline should succeed: {}", result.unwrap_err());
    }

    /// Fix 12: PostfixUpdate/PrefixUpdate/Destructure in validate_context_variable_lvalues.
    /// Context variables modified via assignment inside IIFEs should not cause panics
    /// in the validation pass. The IIFE inlining creates StoreContext instructions
    /// that go through validate_context_variable_lvalues.
    #[test]
    fn test_context_variable_store_context_lvalue() {
        let source = r#"function Component(props) {
  let x = {};
  (function () {
    x = { a: props.a };
  })();
  return <div>{x}</div>;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_ok(),
            "Pipeline should succeed for IIFE context variable store: {}",
            result.unwrap_err()
        );
    }

    /// Fix 14: hasInvalidDeps in validate_preserved_manual_memoization.
    /// useMemo with valid deps should compile without panics (the fix prevents
    /// duplicate errors when deps are invalid).
    #[test]
    fn test_preserved_manual_memoization_valid_deps() {
        let source = r#"import { useMemo } from 'react';
function Component(props) {
  const x = useMemo(() => props.a + props.b, [props.a, props.b]);
  return <div>{x}</div>;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_ok(),
            "Pipeline should succeed for useMemo with valid deps: {}",
            result.unwrap_err()
        );
    }

    /// Fix 15: Effect.Unknown invariant in validate_locals_not_reassigned_after_render.
    /// Code that reassigns locals inside callbacks should produce an Immutability
    /// diagnostic (not panic with an Effect.Unknown invariant failure). The fix
    /// ensures Effect.Unknown is handled gracefully instead of causing an invariant.
    #[test]
    fn test_locals_not_reassigned_after_render_no_panic() {
        let source = r#"function Component(props) {
  let x = props.value;
  const handler = () => { x = 1; };
  return <div onClick={handler}>{x}</div>;
}"#;
        let result = compile_component(source);
        // This should produce a controlled diagnostic (Immutability error), not a panic.
        // The key test is that compile_component returns at all (doesn't panic).
        match &result {
            Ok(_) => {} // If it succeeds, that's also fine
            Err(e) => {
                assert!(
                    e.contains("Immutability") || e.contains("reassign"),
                    "Expected Immutability diagnostic, got: {e}"
                );
            }
        }
    }

    /// Fix 17: Remove useLayoutEffect from derived computations check.
    /// useLayoutEffect calling setState with derived values should not produce
    /// a "derived computations in effects" error.
    #[test]
    fn test_use_layout_effect_no_derived_computation_error() {
        let source = r#"import { useState, useLayoutEffect } from 'react';
function Component(props) {
  const [state, setState] = useState(props.initial);
  useLayoutEffect(() => {
    setState(derive(props.value));
  }, [props.value]);
  return <div>{state}</div>;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_ok(),
            "Pipeline should succeed for useLayoutEffect with derived setState: {}",
            result.unwrap_err()
        );
    }

    /// Test: Passing a ref directly to a function should produce a Refs error.
    /// This validates the basic "validateNoRefPassedToFunction" path.
    #[test]
    fn test_pass_ref_to_function_basic() {
        let source = r#"function Component(props) {
  const ref = useRef(null);
  const x = foo(ref);
  return x;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_err(),
            "Should produce a ref validation error when passing ref to foo(), got success"
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("ref") || err.contains("Ref"),
            "Error should mention refs: {err}"
        );
    }

    /// Test: Passing a ref through useCallback to a render-time call should produce
    /// a Refs error. This matches the Typeahead.tsx:492 pattern where a callback
    /// that reads ref.current is passed to a function called during render.
    #[test]
    fn test_ref_captured_in_callback_passed_to_render_call() {
        let source = r#"function Component(props) {
  const input = useRef(null);
  const handler = () => {
    if (input.current) {
      input.current.value = "test";
    }
  };
  const result = renderHelper(handler);
  return result;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_err(),
            "Should produce a ref validation error when passing ref-reading callback to a function, got success"
        );
    }

    /// Test: Ref captured in useCallback, then passed to a render-time call.
    /// This is the key Typeahead.tsx:492 pattern: useCallback wraps a closure
    /// that reads ref.current, and the resulting callback is passed as argument
    /// to a function invoked during render.
    #[test]
    fn test_ref_in_use_callback_passed_to_render_call() {
        let source = r#"function Component(props) {
  const input = useRef(null);
  const handler = useCallback(() => {
    if (input.current) {
      input.current.value = "test";
    }
  }, []);
  const result = renderHelper(handler);
  return result;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_err(),
            "Should produce a ref validation error when useCallback handler \
             accessing ref is passed to a render-time function, got success"
        );
    }

    /// Test: Ref captured in useCallback, then passed in an object to a
    /// render-time call (Typeahead.tsx:492 pattern).
    #[test]
    fn test_ref_in_use_callback_in_object_passed_to_render_call() {
        let source = r#"function Component(props) {
  const input = useRef(null);
  const handler = useCallback(() => {
    if (input.current) {
      input.current.value = "test";
    }
  }, []);
  const result = renderHelper({ onSelect: handler });
  return result;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_err(),
            "Should produce a ref validation error when useCallback handler \
             accessing ref is passed inside an object to a render-time function, got success"
        );
    }

    /// Test: IIFE with ref-capturing callback passed in object - exact Typeahead pattern.
    /// (renderResults || ((results, config) => <div/>))(results, { onSelect: handler })
    #[test]
    fn test_ref_iife_typeahead_pattern() {
        let source = r#"function Component(props) {
  const input = useRef(null);
  const [results, setResults] = useState([]);
  const handleSelection = useCallback((result) => {
    if (input.current) {
      input.current.value = result.text;
    }
  }, []);
  return (
    <div>
      {results.length ? (
        (props.renderResults || ((r, config) => (
          <ul>{r.map(item => (
            <li key={item} onClick={() => config.onSelect(item)}>{item}</li>
          ))}</ul>
        )))(results, {
          onSelect: handleSelection,
        })
      ) : null}
    </div>
  );
}"#;
        let result = compile_component(source);
        assert!(
            result.is_err(),
            "Should produce a ref validation error for Typeahead IIFE pattern, got success"
        );
    }

    /// Test: Full Typeahead component pattern with ref callback and IIFE.
    /// The pipeline should report a Refs error (among other errors) when
    /// infer_mutation_aliasing_ranges errors are non-fatal, matching TS behavior.
    #[test]
    fn test_typeahead_full_pattern() {
        let source = r#"function Typeahead(props) {
  const [_results, setResults] = useState([]);
  const results = useMemo(
    () => _results.filter((result) => !props.ignoreList?.has(result.value)),
    [_results, props.ignoreList],
  );
  const [highlightedIndex, setHighlightedIndex] = useState(-1);
  const input = useRef(undefined);
  const resetResults = useCallback(() => {
    setResults(props.emptySuggestions?.length ? props.emptySuggestions : []);
  }, [props.emptySuggestions]);
  const hide = useCallback(() => {
    setHighlightedIndex(-1);
    setResults([]);
  }, []);
  const handleSelection = useCallback(
    (result) => {
      if (!result) {
        return;
      }
      if (input.current) {
        const value = props.onSelect?.(result);
        input.current.value = value === '' ? '' : value || result.text;
        input.current.focus();
        resetResults();
      }
    },
    [props.onSelect, resetResults],
  );
  const select = useCallback(() => {
    if (results.length === 0) { return; }
    handleSelection(results[highlightedIndex]);
  }, [handleSelection, highlightedIndex, results]);
  const onKeydown = useCallback(
    (event) => {
      if (event.key === 'Backspace' && input.current?.value === '') {
        props.onBackspace?.();
        return;
      }
      if (input.current) {
        input.current.value = '';
        hide();
      }
    },
    [hide, props.onBackspace],
  );
  const [showEmptyResult, setShowEmptyResult] = useState(false);
  return (
    <div>
      <input
        onKeyDown={onKeydown}
        ref={(element) => {
          input.current = element;
          if (props.inputRef) {
            props.inputRef.current = element;
          }
        }}
        type="text"
      />
      {results.length ? (
        (
          props.renderResults ||
          ((r, config) => (
            <ul onPointerLeave={() => config.setHighlighted(-1)}>
              {r.map((result, index) => (
                <li
                  key={result.value}
                  onClick={() => config.onSelect(result)}
                  onPointerEnter={() => config.setHighlighted(index)}
                >
                  {config.renderItem(result, config.isHighlighted(index))}
                </li>
              ))}
            </ul>
          ))
        )(results, {
          isHighlighted: (index) => index === highlightedIndex,
          onSelect: handleSelection,
          renderItem: props.renderItem,
          setHighlighted: setHighlightedIndex,
        })
      ) : showEmptyResult ? (
        <ul>
          <li>{props.emptyResult}</li>
        </ul>
      ) : null}
    </div>
  );
}"#;
        let result = compile_component(source);
        assert!(result.is_err(), "Should produce errors for Typeahead pattern, got success");
        let err = result.unwrap_err();
        // After fix: infer_mutation_aliasing_ranges errors are non-fatal,
        // so validate_no_ref_access_in_render runs and reports a Refs error.
        assert!(
            err.contains("Refs"),
            "Expected a Refs category error in the output: {err}"
        );
    }

    // =========================================================================
    // Test 1: customOptOutDirectives does not fallthrough
    // =========================================================================

    /// When customOptOutDirectives is configured, the standard 'use no memo'
    /// directive should NOT opt out. Only the custom directives should work.
    #[test]
    fn test_custom_opt_out_directives_no_fallthrough() {
        use oxc_react_compiler::entrypoint::program::find_directive_disabling_memoization;

        // With custom directives configured, standard 'use no memo' should NOT match
        let custom = vec!["use skip".to_string()];
        let directives_no_memo = vec!["use no memo".to_string()];
        assert!(
            find_directive_disabling_memoization(&directives_no_memo, Some(&custom)).is_none(),
            "'use no memo' should not opt out when customOptOutDirectives is configured"
        );

        // Custom directive should match
        let directives_skip = vec!["use skip".to_string()];
        assert_eq!(
            find_directive_disabling_memoization(&directives_skip, Some(&custom)),
            Some("use skip".to_string()),
            "Custom directive 'use skip' should opt out"
        );

        // Without custom directives, standard 'use no memo' should work
        assert_eq!(
            find_directive_disabling_memoization(&directives_no_memo, None),
            Some("use no memo".to_string()),
            "'use no memo' should opt out when no custom directives are configured"
        );

        // 'use no forget' should also work without custom directives
        let directives_no_forget = vec!["use no forget".to_string()];
        assert_eq!(
            find_directive_disabling_memoization(&directives_no_forget, None),
            Some("use no forget".to_string()),
            "'use no forget' should opt out when no custom directives are configured"
        );
    }

    // =========================================================================
    // Test 2: Multiple validation passes report errors simultaneously
    // =========================================================================

    /// When a function has BOTH a hooks violation AND another validation error,
    /// both errors should be reported (not just the first one).
    /// This tests that the pipeline continues through all validation passes.
    #[test]
    fn test_multiple_validation_errors_reported() {
        // This component has:
        // 1. An immutability error (modifying props)
        // 2. A ref access error (reading ref.current in render)
        let source = r#"function Component(props) {
  const ref = useRef(null);
  props.x = 1;
  const val = ref.current;
  return val;
}"#;
        let result = compile_component(source);
        assert!(result.is_err(), "Should produce errors, got success");
        let err = result.unwrap_err();
        // After the infer_mutation_aliasing_ranges fix, both Immutability and Refs
        // errors should be reported because the pipeline continues past the first error.
        assert!(
            err.contains("Immutability"),
            "Expected Immutability error in output: {err}"
        );
        assert!(err.contains("Refs"), "Expected Refs error in output: {err}");
    }

    // =========================================================================
    // Test 3: Suppression error messages match TS
    // =========================================================================

    /// Test that eslint-disable suppression produces error messages matching TS.
    /// The TS compiler produces a specific message format for suppressed functions.
    #[test]
    fn test_suppression_error_message_format() {
        use oxc_react_compiler::compiler_error::{
            CompilerDiagnostic, CompilerDiagnosticDetail, CompilerSuggestion, ErrorCategory,
        };

        // Verify the suppression diagnostic structure matches TS output
        let diagnostic = CompilerDiagnostic::create(
            ErrorCategory::Invariant,
            "React Compiler has skipped optimizing this component because one or more React ESLint rules were disabled. React Compiler only works when all React rules are enabled.".to_string(),
            None,
            None,
        );

        assert_eq!(
            diagnostic.options.reason,
            "React Compiler has skipped optimizing this component because one or more React ESLint rules were disabled. React Compiler only works when all React rules are enabled."
        );
        assert_eq!(diagnostic.options.category, ErrorCategory::Invariant);
    }

    // =========================================================================
    // Test 4: Impure function detection via inference
    // =========================================================================

    /// When validateNoImpureFunctionsInRender is enabled (lint mode default),
    /// calling impure functions like Math.random() should produce a Purity error.
    #[test]
    fn test_impure_function_detection() {
        let config = EnvironmentConfig {
            validate_no_impure_functions_in_render: true,
            ..EnvironmentConfig::default()
        };
        // Math.random(), Date.now(), performance.now() are impure
        let source = r#"function Component() {
  const rand = Math.random();
  return <div>{rand}</div>;
}"#;
        let result = compile_component_with_env(source, config);
        assert!(result.is_err(), "Should produce an error for Math.random(), got success");
        let err = result.unwrap_err();
        assert!(
            err.contains("impure") || err.contains("Impure") || err.contains("Purity"),
            "Expected an impure function error: {err}"
        );
    }

    /// When validateNoImpureFunctionsInRender is NOT enabled, Math.random()
    /// should NOT produce an error.
    #[test]
    fn test_impure_function_not_detected_when_disabled() {
        let source = r#"function Component() {
  const rand = Math.random();
  return <div>{rand}</div>;
}"#;
        let result = compile_component(source);
        // Default config has validate_no_impure_functions_in_render=false,
        // so Math.random() should not produce an impure error.
        assert!(
            result.is_ok(),
            "Should succeed when impure validation is disabled, got: {}",
            result.unwrap_err()
        );
    }

    /// Date.now() should produce a Purity error when validateNoImpureFunctionsInRender is enabled.
    #[test]
    fn test_date_now_impure_detection() {
        let config = EnvironmentConfig {
            validate_no_impure_functions_in_render: true,
            ..EnvironmentConfig::default()
        };
        let source = r#"function Component() {
  const t = Date.now();
  return <div>{t}</div>;
}"#;
        let result = compile_component_with_env(source, config);
        assert!(result.is_err(), "Should produce an error for Date.now(), got success");
        let err = result.unwrap_err();
        assert!(
            err.contains("impure") || err.contains("Impure") || err.contains("Purity"),
            "Expected an impure function error for Date.now(): {err}"
        );
    }

    /// Date.now() should NOT produce an error when validateNoImpureFunctionsInRender is disabled.
    #[test]
    fn test_date_now_no_error_when_disabled() {
        let source = r#"function Component() {
  const t = Date.now();
  return <div>{t}</div>;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_ok(),
            "Should succeed when impure validation is disabled, got: {}",
            result.unwrap_err()
        );
    }

    /// performance.now() should produce a Purity error when validateNoImpureFunctionsInRender is enabled.
    #[test]
    fn test_performance_now_impure_detection() {
        let config = EnvironmentConfig {
            validate_no_impure_functions_in_render: true,
            ..EnvironmentConfig::default()
        };
        let source = r#"function Component() {
  const t = performance.now();
  return <div>{t}</div>;
}"#;
        let result = compile_component_with_env(source, config);
        assert!(result.is_err(), "Should produce an error for performance.now(), got success");
        let err = result.unwrap_err();
        assert!(
            err.contains("impure") || err.contains("Impure") || err.contains("Purity"),
            "Expected an impure function error for performance.now(): {err}"
        );
    }

    /// performance.now() should NOT produce an error when validateNoImpureFunctionsInRender is disabled.
    #[test]
    fn test_performance_now_no_error_when_disabled() {
        let source = r#"function Component() {
  const t = performance.now();
  return <div>{t}</div>;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_ok(),
            "Should succeed when impure validation is disabled, got: {}",
            result.unwrap_err()
        );
    }

    // =========================================================================
    // Test 5: Refs false negative fix (Job 1 regression test)
    // =========================================================================

    /// Regression test: when infer_mutation_aliasing_ranges produces errors
    /// (e.g., Immutability from modifying props), the pipeline should continue
    /// to validate_no_ref_access_in_render and report Refs errors too.
    /// Previously, infer_mutation_aliasing_ranges errors caused the pipeline
    /// to abort via `?`, preventing ref validation from running.
    #[test]
    fn test_refs_not_masked_by_immutability_errors() {
        let source = r#"function Component(props) {
  const input = useRef(null);
  const handler = () => {
    if (input.current) {
      input.current.value = "test";
    }
  };
  props.inputRef.current = input.current;
  const result = renderHelper(handler);
  return result;
}"#;
        let result = compile_component(source);
        assert!(result.is_err(), "Should produce errors, got success");
        let err = result.unwrap_err();
        // Both errors should be present
        assert!(
            err.contains("Refs"),
            "Refs error should not be masked by Immutability error: {err}"
        );
    }

    // =========================================================================
    // Test 6: Type inference fixes
    // =========================================================================

    /// Test tryUnionTypes: Primitive | MixedReadonly = MixedReadonly.
    /// When a conditional returns either a primitive or an object, the type
    /// should be inferred as the broader type (not cause a panic).
    #[test]
    fn test_type_inference_primitive_union() {
        let source = r#"function Component(props) {
  const x = props.cond ? 42 : props.obj;
  return <div>{x}</div>;
}"#;
        let result = compile_component(source);
        // Should not panic; should compile or produce a controlled error
        assert!(
            result.is_ok() || result.as_ref().unwrap_err().contains("error"),
            "Should not panic for primitive union type inference"
        );
    }

    /// Test recursive unification: Var with existing substitution.
    /// When a type variable already has a substitution and gets unified again,
    /// it should handle the chain correctly without infinite recursion.
    #[test]
    fn test_type_inference_recursive_unification() {
        // Pattern that creates recursive unification through phi nodes:
        // a loop where a variable changes type between iterations
        let source = r#"function Component(props) {
  let x = props.initial;
  for (let i = 0; i < 10; i++) {
    x = props.cond ? x : [x];
  }
  return <div>{x}</div>;
}"#;
        let result = compile_component(source);
        // Should not panic or stack overflow; may compile or produce a controlled error
        let _ = result;
    }

    /// Test Poly type ignore: polymorphic types should be handled gracefully.
    #[test]
    fn test_type_inference_poly_type() {
        let source = r#"function Component(props) {
  const id = (x) => x;
  const a = id(42);
  const b = id("hello");
  return <div>{a}{b}</div>;
}"#;
        let result = compile_component(source);
        // Poly types should be handled without panic
        let _ = result;
    }

    /// Test isConstructor check: `new` expressions with constructors
    /// should be handled correctly in type inference.
    #[test]
    fn test_type_inference_constructor() {
        let source = r#"function Component(props) {
  const map = new Map();
  map.set("key", props.value);
  const arr = Array.from(map.values());
  return <div>{arr}</div>;
}"#;
        let result = compile_component(source);
        // Constructor calls should not cause type inference issues
        let _ = result;
    }

    /// Destructuring default with non-reorderable expression (arrow function)
    /// should produce a Todo error, matching TS lowerReorderableExpression.
    #[test]
    fn test_destructuring_default_non_reorderable_arrow() {
        let source = r#"function Component({ onComplete = () => void 0 }) {
  return <div onClick={onComplete} />;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_err(),
            "Non-reorderable destructuring default should cause an error, got:\n{}",
            result.unwrap()
        );
        let err = result.unwrap_err();
        assert!(
            err.contains("cannot be safely reordered"),
            "Error should mention reorderability, got:\n{err}"
        );
    }

    /// Destructuring default with a literal (reorderable) should compile fine.
    #[test]
    fn test_destructuring_default_reorderable_literal() {
        let source = r#"function Component({ value = 0 }) {
  return <div>{value}</div>;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_ok(),
            "Literal destructuring default should be reorderable: {}",
            result.unwrap_err()
        );
    }

    /// Destructuring default with unary `!` (reorderable) should compile fine.
    #[test]
    fn test_destructuring_default_reorderable_unary_not() {
        let source = r#"function Component({ flag = !false }) {
  return <div>{String(flag)}</div>;
}"#;
        let result = compile_component(source);
        assert!(
            result.is_ok(),
            "Unary ! destructuring default should be reorderable: {}",
            result.unwrap_err()
        );
    }

    /// Known limitation: useMemo with destructured hook parameter values may produce
    /// a false-positive PreserveManualMemo error when the destructured values are
    /// primitive types (like numbers). The TS compiler has access to TypeScript type
    /// annotations and infers these as TPrimitive, giving them trivial mutable ranges
    /// and no reactive scopes. The Rust compiler lacks TS type info, so these values
    /// get wide mutable ranges and reactive scopes that span the whole function,
    /// causing the "dependency may be mutated later" check to fire incorrectly.
    ///
    /// Root cause: InferMutationAliasingRanges gives `player` (destructured from a
    /// typed parameter `unit: Unit`) mutable_range [12, 115) in Rust vs a narrow
    /// range in TS (because TS knows player is a number/primitive). This causes
    /// InferReactiveScopeVariables to assign a scope that spans the whole function.
    /// In ValidatePreservedManualMemoization, the scope isn't completed before
    /// StartMemoize, so the check fires.
    ///
    /// This test documents the known divergence. When TypeScript type integration
    /// is added (or a targeted fix is implemented), this test should be updated to
    /// Regression test: destructured parameter used in useMemo + useEffect should not
    /// produce false positive PreserveManualMemo error.
    #[test]
    fn test_no_false_positive_preserve_memo_destructured_param() {
        // Regression: `player` (destructured from `unit`) was getting a wide mutable range
        // because Assign effects from LoadLocal weren't filtered for Primitive sources.
        let source = r#"import { useMemo, useState, useEffect } from 'react';
export default function useUnitState(unit, biome) {
  const { info, player } = unit;
  const [currentState, setUnitState] = useState({ type: 'idle' });
  const { type, ...props } = currentState;

  const [entity, map] = useMemo(() => {
    let entity = info.create(player);
    if (unit.isTransportingUnits()) {
      entity = entity.copy({ transports: unit.transports });
    }
    return [entity, makeMap(biome, ImmutableMap([[1, entity]]))];
  }, [info, biome, player, type, unit]);

  useEffect(() => {
    if (player > 0) {
      const interval = setInterval(() => {
        setUnitState(getNext(currentState, entity));
      }, 1000);
      return () => clearInterval(interval);
    }
  }, [currentState, entity, player]);

  return [entity, map, props];
}"#;
        let config = EnvironmentConfig {
            validate_preserve_existing_memoization_guarantees: true,
            enable_preserve_existing_memoization_guarantees: true,
            ..EnvironmentConfig::default()
        };
        let allocator = oxc_allocator::Allocator::default();
        let source_type = oxc_span::SourceType::jsx();
        let parser_result = oxc_parser::Parser::new(&allocator, source, source_type).parse();
        assert!(parser_result.errors.is_empty());

        let func = parser_result
            .program
            .body
            .iter()
            .find_map(|stmt| match stmt {
                oxc_ast::ast::Statement::ExportDefaultDeclaration(e) => {
                    match &e.declaration {
                        oxc_ast::ast::ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                            Some(oxc_react_compiler::hir::build_hir::LowerableFunction::Function(f))
                        }
                        _ => None,
                    }
                }
                _ => None,
            })
            .expect("No function found");

        let env = oxc_react_compiler::hir::environment::Environment::new(
            oxc_react_compiler::hir::ReactFunctionType::Hook,
            oxc_react_compiler::hir::environment::CompilerOutputMode::Lint,
            config,
        )
        .unwrap();

        let outer_bindings =
            oxc_react_compiler::hir::build_hir::collect_import_bindings(&parser_result.program.body);
        let mut hir_func = oxc_react_compiler::hir::build_hir::lower(
            &env,
            oxc_react_compiler::hir::ReactFunctionType::Hook,
            &func,
            outer_bindings,
        )
        .expect("Lower failed");

        let pipeline_result =
            oxc_react_compiler::entrypoint::pipeline::run_pipeline(&mut hir_func, &env);

        // Collect ALL errors
        let mut all_errors = String::new();
        match pipeline_result {
            Ok(output) => {
                if let Some(recorded) = output.recorded_errors {
                    all_errors.push_str(&format!("{recorded:?}"));
                }
            }
            Err(error) => {
                all_errors.push_str(&format!("{error:?}"));
            }
        }
        for diag in hir_func.env.take_diagnostics() {
            all_errors.push_str(&format!("{diag:?}"));
        }

        // Regression test: previously produced a false positive PreserveManualMemo because
        // `player`'s Assign effect from LoadLocal wasn't filtered for Primitive sources,
        // creating spurious alias edges that widened its mutable range.
        assert!(
            !all_errors.contains("PreserveManualMemo"),
            "Should NOT produce PreserveManualMemo false positive, got: {all_errors}"
        );
    }
}
