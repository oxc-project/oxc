//! Fixture configuration shared by compiler snapshots and Sprout.

use convert_case::{Case, Casing};
use oxc_react_compiler::{
    AliasingEffectConfig, AliasingSignatureConfig, BuiltInTypeRef, CompilationMode,
    DynamicGatingConfig, Effect, EnvironmentConfig, ExhaustiveEffectDepsMode,
    ExternalFunctionConfig, FunctionTypeConfig, FxIndexMap, GatingConfig, HookTypeConfig,
    InstrumentationConfig, ObjectTypeConfig, PanicThreshold, PluginOptions, TypeConfig,
    TypeReferenceConfig, ValueKind, ValueReason,
};
use oxc_span::SourceType;

/// Build the per-fixture `SourceType` + `PluginOptions` from the first-line pragmas.
pub fn parse_pragma(source: &str) -> (SourceType, PluginOptions) {
    // Upstream `snap` defaults: compile everything, surface every error.
    let mut options = PluginOptions {
        compilation_mode: CompilationMode::All,
        panic_threshold: PanicThreshold::AllErrors,
        ..PluginOptions::default()
    };
    // Mirror the snap runner's test `moduleTypeProvider`; unlisted modules fall back to
    // oxc's default provider. Include Sprout helpers and the magic validation modules.
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
                if let Some(v) = value.and_then(|v| unquote(v).parse().ok()) {
                    options.compilation_mode = v;
                }
            }
            "panicThreshold" => {
                if let Some(v) = value.and_then(|v| unquote(v).parse().ok()) {
                    options.panic_threshold = v;
                }
            }
            "outputMode" => options.output_mode = value.and_then(|v| unquote(v).parse().ok()),
            "lintSkipReactiveScopePasses" => options.lint_skip_reactive_scope_passes = true,
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

    // Mirror the snap runner (`packages/snap/src/compiler.ts`): the fixture corpus runs
    // with `validatePreserveExistingMemoizationGuarantees` OFF by default — most fixtures
    // care about compilation output, not whether manual memoization is preserved — and it
    // is turned on only when the fixture's first line opts in via the directive. Snap keys
    // off substring presence alone (ignoring any `:value`), so replicate that exactly and
    // let it override whatever the directive loop parsed. Without this, fixtures that set
    // `@enablePreserveExistingMemoizationGuarantees:false` still hit `ValidatePreservedManual-
    // Memoization` and bail with a spurious "memoization could not be preserved" error.
    options.environment.validate_preserve_existing_memoization_guarantees = source
        .lines()
        .next()
        .unwrap_or("")
        .contains("@validatePreserveExistingMemoizationGuarantees");

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
    value.strip_circumfix('"', '"').unwrap_or(value).to_string()
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
            env.validate_no_capitalized_calls = Some(value.map_or_default(json_strings));
        }
        "validate_blocklisted_imports" => {
            env.validate_blocklisted_imports = Some(value.map_or_default(json_strings));
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

/// Sprout's runtime signatures plus the magic modules used by type-provider and
/// known-incompatible-library validation fixtures.
fn test_module_type_provider() -> FxIndexMap<String, TypeConfig> {
    FxIndexMap::from_iter([
        ("shared-runtime".to_string(), shared_runtime_type_provider()),
        (
            "ReactCompilerKnownIncompatibleTest".to_string(),
            object([
                (
                    "useKnownIncompatible",
                    hook(
                        type_ref(),
                        Some(Vec::new()),
                        Some("useKnownIncompatible is known to be incompatible".to_string()),
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
                (
                    "knownIncompatibleAliasing",
                    incompatible_aliasing_fn(
                        "knownIncompatibleAliasing is known to be incompatible",
                    ),
                ),
            ]),
        ),
        (
            "ReactCompilerNoAliasTest".to_string(),
            object([("applyCallbackAndReturnPrimitive", no_alias_apply_primitive_fn())]),
        ),
        (
            "ReactCompilerTest".to_string(),
            object([
                ("useHookNotTypedAsHook", type_ref()),
                ("notAhookTypedAsHook", hook(type_ref(), None, None)),
            ]),
        ),
        (
            "ReactCompilerPureTagTest".to_string(),
            object([(
                "tag",
                TypeConfig::Function(FunctionTypeConfig {
                    positional_params: Vec::new(),
                    rest_param: Some(Effect::Read),
                    callee_effect: Effect::Read,
                    return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                        name: BuiltInTypeRef::Any,
                    })),
                    return_value_kind: ValueKind::Mutable,
                    no_alias: None,
                    mutable_only_if_operands_are_mutable: None,
                    impure: Some(false),
                    canonical_name: None,
                    aliasing: None,
                    known_incompatible: None,
                }),
            )]),
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

fn no_alias_apply_primitive_fn() -> TypeConfig {
    TypeConfig::Function(FunctionTypeConfig {
        positional_params: vec![Effect::ConditionallyMutate],
        rest_param: None,
        callee_effect: Effect::Read,
        return_type: Box::new(type_ref()),
        return_value_kind: ValueKind::Primitive,
        no_alias: Some(true),
        mutable_only_if_operands_are_mutable: None,
        impure: None,
        canonical_name: None,
        aliasing: Some(AliasingSignatureConfig {
            receiver: "@receiver",
            params: &["@callback"],
            rest: None,
            returns: "@returns",
            temporaries: &["@callbackReturn"],
            effects: &[
                AliasingEffectConfig::Create {
                    into: "@returns",
                    value: ValueKind::Primitive,
                    reason: ValueReason::KnownReturnSignature,
                },
                AliasingEffectConfig::Apply {
                    receiver: "@receiver",
                    function: "@callback",
                    mutates_function: false,
                    args: &[],
                    into: "@callbackReturn",
                },
            ],
        }),
        known_incompatible: None,
    })
}

fn incompatible_aliasing_fn(message: &str) -> TypeConfig {
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
        aliasing: Some(AliasingSignatureConfig {
            receiver: "@receiver",
            params: &[],
            rest: Some("@rest"),
            returns: "@returns",
            temporaries: &[],
            effects: &[AliasingEffectConfig::Create {
                into: "@returns",
                value: ValueKind::Mutable,
                reason: ValueReason::KnownReturnSignature,
            }],
        }),
        known_incompatible: Some(message.to_string()),
    })
}

/// Ported from snap/src/sprout/shared-runtime-type-provider.ts.
fn shared_runtime_type_provider() -> TypeConfig {
    object([
        (
            "default",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![],
                rest_param: Some(Effect::Read),
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Primitive,
                })),
                return_value_kind: ValueKind::Primitive,
                no_alias: None,
                aliasing: None,
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "graphql",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![],
                rest_param: Some(Effect::Read),
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Primitive,
                })),
                return_value_kind: ValueKind::Primitive,
                no_alias: None,
                aliasing: None,
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "typedArrayPush",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![Effect::Store, Effect::Capture],
                rest_param: Some(Effect::Capture),
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Primitive,
                })),
                return_value_kind: ValueKind::Primitive,
                no_alias: None,
                aliasing: None,
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "typedLog",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![],
                rest_param: Some(Effect::Read),
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Primitive,
                })),
                return_value_kind: ValueKind::Primitive,
                no_alias: None,
                aliasing: None,
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "useFreeze",
            TypeConfig::Hook(HookTypeConfig {
                positional_params: None,
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Any,
                })),
                return_value_kind: None,
                no_alias: None,
                aliasing: None,
                known_incompatible: None,
            }),
        ),
        (
            "useFragment",
            TypeConfig::Hook(HookTypeConfig {
                positional_params: None,
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::MixedReadonly,
                })),
                return_value_kind: None,
                no_alias: Some(true),
                aliasing: None,
                known_incompatible: None,
            }),
        ),
        (
            "useNoAlias",
            TypeConfig::Hook(HookTypeConfig {
                positional_params: None,
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Any,
                })),
                return_value_kind: Some(ValueKind::Mutable),
                no_alias: Some(true),
                aliasing: None,
                known_incompatible: None,
            }),
        ),
        (
            "typedIdentity",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![Effect::Read],
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Any,
                })),
                return_value_kind: ValueKind::Mutable,
                no_alias: None,
                aliasing: Some(AliasingSignatureConfig {
                    receiver: "@receiver",
                    params: &["@value"],
                    rest: None,
                    returns: "@return",
                    temporaries: &[],
                    effects: &[AliasingEffectConfig::Assign { from: "@value", into: "@return" }],
                }),
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "typedAssign",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![Effect::Read],
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Any,
                })),
                return_value_kind: ValueKind::Mutable,
                no_alias: None,
                aliasing: Some(AliasingSignatureConfig {
                    receiver: "@receiver",
                    params: &["@value"],
                    rest: None,
                    returns: "@return",
                    temporaries: &[],
                    effects: &[AliasingEffectConfig::Assign { from: "@value", into: "@return" }],
                }),
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "typedAlias",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![Effect::Read],
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Any,
                })),
                return_value_kind: ValueKind::Mutable,
                no_alias: None,
                aliasing: Some(AliasingSignatureConfig {
                    receiver: "@receiver",
                    params: &["@value"],
                    rest: None,
                    returns: "@return",
                    temporaries: &[],
                    effects: &[
                        AliasingEffectConfig::Create {
                            into: "@return",
                            value: ValueKind::Mutable,
                            reason: ValueReason::KnownReturnSignature,
                        },
                        AliasingEffectConfig::Alias { from: "@value", into: "@return" },
                    ],
                }),
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "typedCapture",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![Effect::Read],
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Array,
                })),
                return_value_kind: ValueKind::Mutable,
                no_alias: None,
                aliasing: Some(AliasingSignatureConfig {
                    receiver: "@receiver",
                    params: &["@value"],
                    rest: None,
                    returns: "@return",
                    temporaries: &[],
                    effects: &[
                        AliasingEffectConfig::Create {
                            into: "@return",
                            value: ValueKind::Mutable,
                            reason: ValueReason::KnownReturnSignature,
                        },
                        AliasingEffectConfig::Capture { from: "@value", into: "@return" },
                    ],
                }),
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "typedCreateFrom",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![Effect::Read],
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Any,
                })),
                return_value_kind: ValueKind::Mutable,
                no_alias: None,
                aliasing: Some(AliasingSignatureConfig {
                    receiver: "@receiver",
                    params: &["@value"],
                    rest: None,
                    returns: "@return",
                    temporaries: &[],
                    effects: &[AliasingEffectConfig::CreateFrom {
                        from: "@value",
                        into: "@return",
                    }],
                }),
                known_incompatible: None,
                callee_effect: Effect::Read,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "typedMutate",
            TypeConfig::Function(FunctionTypeConfig {
                positional_params: vec![Effect::Read, Effect::Capture],
                rest_param: None,
                return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                    name: BuiltInTypeRef::Primitive,
                })),
                return_value_kind: ValueKind::Primitive,
                no_alias: None,
                aliasing: Some(AliasingSignatureConfig {
                    receiver: "@receiver",
                    params: &["@object", "@value"],
                    rest: None,
                    returns: "@return",
                    temporaries: &[],
                    effects: &[
                        AliasingEffectConfig::Create {
                            into: "@return",
                            value: ValueKind::Primitive,
                            reason: ValueReason::KnownReturnSignature,
                        },
                        AliasingEffectConfig::Mutate { value: "@object" },
                        AliasingEffectConfig::Capture { from: "@value", into: "@object" },
                    ],
                }),
                known_incompatible: None,
                callee_effect: Effect::Store,
                mutable_only_if_operands_are_mutable: None,
                impure: None,
                canonical_name: None,
            }),
        ),
        (
            "PanResponder",
            object([(
                "create",
                TypeConfig::Function(FunctionTypeConfig {
                    positional_params: vec![Effect::Freeze],
                    rest_param: None,
                    return_type: Box::new(TypeConfig::TypeReference(TypeReferenceConfig {
                        name: BuiltInTypeRef::Any,
                    })),
                    return_value_kind: ValueKind::Frozen,
                    no_alias: None,
                    aliasing: Some(AliasingSignatureConfig {
                        receiver: "@receiver",
                        params: &["@config"],
                        rest: None,
                        returns: "@returns",
                        temporaries: &[],
                        effects: &[
                            AliasingEffectConfig::Freeze {
                                value: "@config",
                                reason: ValueReason::KnownReturnSignature,
                            },
                            AliasingEffectConfig::Create {
                                into: "@returns",
                                value: ValueKind::Frozen,
                                reason: ValueReason::KnownReturnSignature,
                            },
                            AliasingEffectConfig::ImmutableCapture {
                                from: "@config",
                                into: "@returns",
                            },
                        ],
                    }),
                    known_incompatible: None,
                    callee_effect: Effect::Read,
                    mutable_only_if_operands_are_mutable: None,
                    impure: None,
                    canonical_name: None,
                }),
            )]),
        ),
    ])
}
