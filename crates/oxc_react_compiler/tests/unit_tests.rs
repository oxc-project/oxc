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
        let result = with_parsed_function(
            "function Component() { return <div />; }",
            |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Infer, false, false, None)
            },
        );
        assert_eq!(result, Some(ReactFunctionType::Component));
    }

    #[test]
    fn should_compile_hook() {
        // Hook with a hook call — qualifies as hook in Infer mode
        let result = with_parsed_function(
            "function useMyHook() { return useState(0); }",
            |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Infer, false, false, None)
            },
        );
        assert_eq!(result, Some(ReactFunctionType::Hook));
    }

    #[test]
    fn should_not_compile_regular_function() {
        let result = with_parsed_function("function helper() { return 1; }", |func, name| {
            should_compile_function(func, name, &[], CompilationMode::Infer, false, false, None)
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
                None,
            )
        });
        assert!(result.is_some());
    }

    #[test]
    fn should_not_compile_with_use_no_memo_directive() {
        let result = with_parsed_function(
            "function Component() { return <div />; }",
            |func, name| {
                should_compile_function(
                    func,
                    name,
                    &["use no memo".to_string()],
                    CompilationMode::Infer,
                    false,
                    false,
                    None,
                )
            },
        );
        assert_eq!(result, None);
    }

    #[test]
    fn annotation_mode_requires_directive() {
        let result = with_parsed_function(
            "function Component() { return <div />; }",
            |func, name| {
                should_compile_function(func, name, &[], CompilationMode::Annotation, false, false, None)
            },
        );
        assert_eq!(result, None);

        let result = with_parsed_function(
            "function Component() { return <div />; }",
            |func, name| {
                should_compile_function(
                    func,
                    name,
                    &["use memo".to_string()],
                    CompilationMode::Annotation,
                    false,
                    false,
                    None,
                )
            },
        );
        assert!(result.is_some());
    }

    #[test]
    fn all_mode_compiles_everything() {
        // Even a plain helper is compiled in All mode (as Other)
        let result = with_parsed_function("function helper() { return 1; }", |func, name| {
            should_compile_function(func, name, &[], CompilationMode::All, false, false, None)
        });
        assert!(result.is_some());
    }

    #[test]
    fn memo_callback_compiles_as_component_in_infer_mode() {
        // An anonymous-like function inside React.memo() should compile as Component
        // if it calls hooks or creates JSX. Pass is_memo_or_forwardref_arg=true.
        let result = with_parsed_function(
            "function _temp() { return <div />; }",
            |func, _name| {
                // Pass None for name to simulate anonymous, and is_memo_or_forwardref_arg=true
                should_compile_function(func, None, &[], CompilationMode::Infer, true, false, None)
            },
        );
        assert_eq!(result, Some(ReactFunctionType::Component));
    }

    #[test]
    fn memo_callback_compiles_as_component_in_all_mode() {
        let result = with_parsed_function(
            "function _temp() { return <div />; }",
            |func, _name| {
                should_compile_function(func, None, &[], CompilationMode::All, true, false, None)
            },
        );
        assert_eq!(result, Some(ReactFunctionType::Component));
    }

    #[test]
    fn forwardref_callback_not_compiled_without_flag() {
        // Without is_memo_or_forwardref_arg, an anonymous function should not compile in Infer mode
        let result = with_parsed_function(
            "function _temp() { return <div />; }",
            |func, _name| {
                should_compile_function(func, None, &[], CompilationMode::Infer, false, false, None)
            },
        );
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
    );

    let mut hir_func =
        lower(&env, ReactFunctionType::Component, &func, rustc_hash::FxHashMap::default())
            .expect("Lower failed");
    let pipeline_output = run_pipeline(&mut hir_func, &env).expect("Pipeline failed");
    let ast = oxc_ast::AstBuilder::new(&allocator);
    let result = run_codegen(pipeline_output, &env, ast, "_c").expect("Codegen failed");
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
    );

    let mut hir_func =
        lower(&env, ReactFunctionType::Component, &func, rustc_hash::FxHashMap::default())
            .expect("Lower failed");
    let pipeline_output = run_pipeline(&mut hir_func, &env).expect("Pipeline failed");
    let ast = oxc_ast::AstBuilder::new(&allocator);
    let result = run_codegen(pipeline_output, &env, ast, "_c").expect("Codegen failed");

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

    let env = Environment::new(
        ReactFunctionType::Component,
        CompilerOutputMode::Client,
        EnvironmentConfig::default(),
    );

    // Helper to verify a console-like object has a "log" method with Read effects.
    let verify_console_log =
        |console_type: &oxc_react_compiler::hir::types::Type, label: &str| match console_type {
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
    verify_console_log(&console_type, "console");

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
    verify_console_log(&global_console_type.unwrap(), "global.console");

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
    verify_console_log(&global_this_console_type.unwrap(), "globalThis.console");
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
        Environment::new(ReactFunctionType::Component, CompilerOutputMode::Client, env_config);

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
            let codegen_func = run_codegen(pipeline_output, &env, ast, "_c").expect("Codegen failed");
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
