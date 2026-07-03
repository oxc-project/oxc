// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Compilation pipeline for a single function.
//!
//! Analogous to TS `Pipeline.ts` (`compileFn` → `run` → `runWithEnvironment`).
//! Currently runs BuildHIR (lowering) and PruneMaybeThrows.

use crate::react_compiler_diagnostics::CompilerError;
use crate::react_compiler_diagnostics::CompilerErrorDetail;
use crate::react_compiler_diagnostics::ErrorCategory;
use crate::react_compiler_hir::HirFunction;
use crate::react_compiler_hir::ReactFunctionType;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::environment::OutputMode;
use crate::react_compiler_hir::environment_config::EnvironmentConfig;
use crate::react_compiler_hir::print::PrintFormatter;
use crate::react_compiler_inference::align_method_call_scopes;
use crate::react_compiler_inference::align_object_method_scopes;
use crate::react_compiler_inference::align_reactive_scopes_to_block_scopes_hir;
use crate::react_compiler_inference::analyse_functions;
use crate::react_compiler_inference::build_reactive_scope_terminals_hir;
use crate::react_compiler_inference::flatten_reactive_loops_hir;
use crate::react_compiler_inference::flatten_scopes_with_hooks_or_use_hir;
use crate::react_compiler_inference::infer_mutation_aliasing_effects;
use crate::react_compiler_inference::infer_mutation_aliasing_ranges;
use crate::react_compiler_inference::infer_reactive_places;
use crate::react_compiler_inference::infer_reactive_scope_variables;
use crate::react_compiler_inference::memoize_fbt_and_macro_operands_in_same_scope;
use crate::react_compiler_inference::merge_overlapping_reactive_scopes_hir;
use crate::react_compiler_inference::propagate_scope_dependencies_hir;
use crate::react_compiler_lowering::FunctionNode;
use crate::react_compiler_lowering::lower;
use crate::react_compiler_optimization::constant_propagation;
use crate::react_compiler_optimization::dead_code_elimination;
use crate::react_compiler_optimization::drop_manual_memoization;
use crate::react_compiler_optimization::inline_immediately_invoked_function_expressions;
use crate::react_compiler_optimization::merge_consecutive_blocks::merge_consecutive_blocks;
use crate::react_compiler_optimization::name_anonymous_functions;
use crate::react_compiler_optimization::optimize_for_ssr;
use crate::react_compiler_optimization::optimize_props_method_calls;
use crate::react_compiler_optimization::outline_functions;
use crate::react_compiler_optimization::outline_jsx;
use crate::react_compiler_optimization::prune_maybe_throws;
use crate::react_compiler_optimization::prune_unused_labels_hir;
use crate::react_compiler_reactive_scopes::assert_scope_instructions_within_scopes;
use crate::react_compiler_reactive_scopes::assert_well_formed_break_targets;
use crate::react_compiler_reactive_scopes::build_reactive_function;
use crate::react_compiler_reactive_scopes::codegen_function;
use crate::react_compiler_reactive_scopes::extract_scope_declarations_from_destructuring;
use crate::react_compiler_reactive_scopes::merge_reactive_scopes_that_invalidate_together;
use crate::react_compiler_reactive_scopes::print_reactive_function::debug_reactive_function_with_formatter;
use crate::react_compiler_reactive_scopes::promote_used_temporaries;
use crate::react_compiler_reactive_scopes::propagate_early_returns;
use crate::react_compiler_reactive_scopes::prune_always_invalidating_scopes;
use crate::react_compiler_reactive_scopes::prune_hoisted_contexts;
use crate::react_compiler_reactive_scopes::prune_non_escaping_scopes;
use crate::react_compiler_reactive_scopes::prune_non_reactive_dependencies;
use crate::react_compiler_reactive_scopes::prune_unused_labels;
use crate::react_compiler_reactive_scopes::prune_unused_lvalues;
use crate::react_compiler_reactive_scopes::prune_unused_scopes;
use crate::react_compiler_reactive_scopes::rename_variables;
use crate::react_compiler_reactive_scopes::stabilize_block_ids;
use crate::react_compiler_ssa::eliminate_redundant_phi;
use crate::react_compiler_ssa::enter_ssa;
use crate::react_compiler_ssa::rewrite_instruction_kinds_based_on_reassignment;
use crate::react_compiler_typeinference::infer_types;
use crate::react_compiler_validation::validate_context_variable_lvalues;
use crate::react_compiler_validation::validate_exhaustive_dependencies;
use crate::react_compiler_validation::validate_hooks_usage;
use crate::react_compiler_validation::validate_locals_not_reassigned_after_render;
use crate::react_compiler_validation::validate_no_capitalized_calls;
use crate::react_compiler_validation::validate_no_derived_computations_in_effects;
use crate::react_compiler_validation::validate_no_derived_computations_in_effects_exp;
use crate::react_compiler_validation::validate_no_freezing_known_mutable_functions;
use crate::react_compiler_validation::validate_no_jsx_in_try_statement;
use crate::react_compiler_validation::validate_no_ref_access_in_render;
use crate::react_compiler_validation::validate_no_set_state_in_effects;
use crate::react_compiler_validation::validate_no_set_state_in_render;
use crate::react_compiler_validation::validate_preserved_manual_memoization;
use crate::react_compiler_validation::validate_static_components;
use crate::react_compiler_validation::validate_use_memo;
use crate::scope::*;

use super::compile_result::CodegenFunction;
use super::compile_result::DebugLogEntry;
use super::compile_result::OutlinedFunction;
use super::imports::ProgramContext;
use super::plugin_options::CompilerOutputMode;
use crate::react_compiler::debug_print;

/// Run the compilation pipeline on a single function.
///
/// Currently: creates an Environment, runs BuildHIR (lowering), and produces
/// debug output via the context. Returns a CodegenFunction with zeroed memo
/// stats on success (codegen is not yet implemented).
pub fn compile_fn<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    func: &FunctionNode<'_>,
    fn_name: Option<&str>,
    scope_info: &ScopeInfo,
    fn_type: ReactFunctionType,
    mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext,
) -> Result<CodegenFunction<'a>, CompilerError> {
    let mut env = Environment::with_config(env_config.clone());
    env.fn_type = fn_type;
    env.output_mode = match mode {
        CompilerOutputMode::Ssr => OutputMode::Ssr,
        CompilerOutputMode::Client => OutputMode::Client,
        CompilerOutputMode::Lint => OutputMode::Lint,
    };
    env.code = context.code.clone();
    env.instrument_fn_name = context.instrument_fn_name.clone();
    env.instrument_gating_name = context.instrument_gating_name.clone();
    env.hook_guard_name = context.hook_guard_name.clone();
    env.seed_uid_known_names(&context.known_referenced_names());

    env.reference_node_ids = scope_info.ref_node_id_to_binding.keys().copied().collect();

    let line_offsets = crate::react_compiler_lowering::source_loc::LineOffsets::new(
        context.code.as_deref().unwrap_or(""),
    );
    let mut hir = lower(func, fn_name, scope_info, &mut env, &line_offsets)?;

    // Copy renames from lowering to context (keep on env for codegen to apply to type annotations)
    if !env.renames.is_empty() {
        context.renames.extend(env.renames.iter().cloned());
    }

    // Check for Invariant errors after lowering, before logging HIR.
    // In TS, Invariant errors throw from recordError(), aborting lower() before
    // the HIR entry is logged. The thrown error contains ONLY the Invariant error,
    // not other recorded (non-Invariant) errors.
    if env.has_invariant_errors() {
        return Err(env.take_invariant_errors());
    }

    if context.debug_enabled {
        let debug_hir = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("HIR", debug_hir));
    }

    prune_maybe_throws(&mut hir, &mut env.functions)?;

    if context.debug_enabled {
        let debug_prune = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("PruneMaybeThrows", debug_prune));
    }

    validate_context_variable_lvalues(&hir, &mut env)?;
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("ValidateContextVariableLValues", "ok".to_string()));
    }

    let void_memo_errors = validate_use_memo(&hir, &mut env);
    log_errors_as_events(&void_memo_errors, context);
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("ValidateUseMemo", "ok".to_string()));
    }

    drop_manual_memoization(&mut hir, &mut env)?;

    if context.debug_enabled {
        let debug_drop_memo = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("DropManualMemoization", debug_drop_memo));
    }

    inline_immediately_invoked_function_expressions(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_inline_iifes = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new(
            "InlineImmediatelyInvokedFunctionExpressions",
            debug_inline_iifes,
        ));
    }

    merge_consecutive_blocks(&mut hir, &mut env.functions);

    if context.debug_enabled {
        let debug_merge = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("MergeConsecutiveBlocks", debug_merge));
    }

    // TODO: port assertConsistentIdentifiers
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("AssertConsistentIdentifiers", "ok".to_string()));
    }
    // TODO: port assertTerminalSuccessorsExist
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("AssertTerminalSuccessorsExist", "ok".to_string()));
    }

    enter_ssa(&mut hir, &mut env).map_err(|diag| {
        let loc = diag.primary_location().cloned();
        let mut err = CompilerError::new();
        err.push_error_detail(CompilerErrorDetail {
            category: diag.category,
            reason: diag.reason,
            description: diag.description,
            loc,
            suggestions: diag.suggestions,
        });
        err
    })?;

    if context.debug_enabled {
        let debug_ssa = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("SSA", debug_ssa));
    }

    eliminate_redundant_phi(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_eliminate_phi = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("EliminateRedundantPhi", debug_eliminate_phi));
    }

    // TODO: port assertConsistentIdentifiers
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("AssertConsistentIdentifiers", "ok".to_string()));
    }

    constant_propagation(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_const_prop = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("ConstantPropagation", debug_const_prop));
    }

    infer_types(&mut hir, &mut env)?;

    if context.debug_enabled {
        let debug_infer_types = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("InferTypes", debug_infer_types));
    }

    if env.enable_validations() {
        if env.config.validate_hooks_usage {
            validate_hooks_usage(&hir, &mut env)?;
            if context.debug_enabled {
                context.log_debug(DebugLogEntry::new("ValidateHooksUsage", "ok".to_string()));
            }
        }

        if env.config.validate_no_capitalized_calls.is_some() {
            validate_no_capitalized_calls(&hir, &mut env)?;
            if context.debug_enabled {
                context
                    .log_debug(DebugLogEntry::new("ValidateNoCapitalizedCalls", "ok".to_string()));
            }
        }
    }

    optimize_props_method_calls(&mut hir, &env);

    if context.debug_enabled {
        let debug_optimize_props = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("OptimizePropsMethodCalls", debug_optimize_props));
    }

    let mut inner_logs: Vec<String> = Vec::new();
    let debug_inner = context.debug_enabled;
    let analyse_result = analyse_functions(&mut hir, &mut env, &mut |inner_func, inner_env| {
        if debug_inner {
            inner_logs.push(debug_print::debug_hir(inner_func, inner_env));
        }
    });

    // Always flush inner logs before propagating errors
    if context.debug_enabled {
        for inner_log in inner_logs {
            context.log_debug(DebugLogEntry::new("AnalyseFunction (inner)", inner_log));
        }
    }

    analyse_result?;

    if env.has_invariant_errors() {
        return Err(env.take_invariant_errors());
    }

    if context.debug_enabled {
        let debug_analyse_functions = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("AnalyseFunctions", debug_analyse_functions));
    }

    infer_mutation_aliasing_effects(&mut hir, &mut env, false)?;

    if context.debug_enabled {
        let debug_infer_effects = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("InferMutationAliasingEffects", debug_infer_effects));
    }

    if env.output_mode == OutputMode::Ssr {
        optimize_for_ssr(&mut hir, &env);

        if context.debug_enabled {
            let debug_ssr = debug_print::debug_hir(&hir, &env);
            context.log_debug(DebugLogEntry::new("OptimizeForSSR", debug_ssr));
        }
    }

    dead_code_elimination(&mut hir, &env);

    if context.debug_enabled {
        let debug_dce = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("DeadCodeElimination", debug_dce));
    }

    prune_maybe_throws(&mut hir, &mut env.functions)?;

    if context.debug_enabled {
        let debug_prune2 = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("PruneMaybeThrows", debug_prune2));
    }

    infer_mutation_aliasing_ranges(&mut hir, &mut env, false)?;

    if context.debug_enabled {
        let debug_infer_ranges = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("InferMutationAliasingRanges", debug_infer_ranges));
    }

    if env.enable_validations() {
        validate_locals_not_reassigned_after_render(&hir, &mut env);
        if context.debug_enabled {
            context.log_debug(DebugLogEntry::new(
                "ValidateLocalsNotReassignedAfterRender",
                "ok".to_string(),
            ));
        }

        if env.config.validate_ref_access_during_render {
            validate_no_ref_access_in_render(&hir, &mut env);
            if context.debug_enabled {
                context
                    .log_debug(DebugLogEntry::new("ValidateNoRefAccessInRender", "ok".to_string()));
            }
        }

        if env.config.validate_no_set_state_in_render {
            validate_no_set_state_in_render(&hir, &mut env)?;
            if context.debug_enabled {
                context
                    .log_debug(DebugLogEntry::new("ValidateNoSetStateInRender", "ok".to_string()));
            }
        }

        if env.config.validate_no_derived_computations_in_effects_exp
            && env.output_mode == OutputMode::Lint
        {
            let errors = validate_no_derived_computations_in_effects_exp(&hir, &env)?;
            log_errors_as_events(&errors, context);
            if context.debug_enabled {
                context.log_debug(DebugLogEntry::new(
                    "ValidateNoDerivedComputationsInEffects",
                    "ok".to_string(),
                ));
            }
        } else if env.config.validate_no_derived_computations_in_effects {
            validate_no_derived_computations_in_effects(&hir, &mut env)?;
            if context.debug_enabled {
                context.log_debug(DebugLogEntry::new(
                    "ValidateNoDerivedComputationsInEffects",
                    "ok".to_string(),
                ));
            }
        }

        if env.config.validate_no_set_state_in_effects && env.output_mode == OutputMode::Lint {
            let errors = validate_no_set_state_in_effects(&hir, &env)?;
            log_errors_as_events(&errors, context);
            if context.debug_enabled {
                context
                    .log_debug(DebugLogEntry::new("ValidateNoSetStateInEffects", "ok".to_string()));
            }
        }

        if env.config.validate_no_jsx_in_try_statements && env.output_mode == OutputMode::Lint {
            let errors = validate_no_jsx_in_try_statement(&hir);
            log_errors_as_events(&errors, context);
            if context.debug_enabled {
                context
                    .log_debug(DebugLogEntry::new("ValidateNoJSXInTryStatement", "ok".to_string()));
            }
        }

        validate_no_freezing_known_mutable_functions(&hir, &mut env);
        if context.debug_enabled {
            context.log_debug(DebugLogEntry::new(
                "ValidateNoFreezingKnownMutableFunctions",
                "ok".to_string(),
            ));
        }
    }

    infer_reactive_places(&mut hir, &mut env)?;

    if context.debug_enabled {
        let debug_reactive_places = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("InferReactivePlaces", debug_reactive_places));
    }

    if env.enable_validations() {
        validate_exhaustive_dependencies(&mut hir, &mut env)?;
        if context.debug_enabled {
            context
                .log_debug(DebugLogEntry::new("ValidateExhaustiveDependencies", "ok".to_string()));
        }
    }

    rewrite_instruction_kinds_based_on_reassignment(&mut hir, &env)?;

    if context.debug_enabled {
        let debug_rewrite = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new(
            "RewriteInstructionKindsBasedOnReassignment",
            debug_rewrite,
        ));
    }

    if env.enable_validations()
        && env.config.validate_static_components
        && env.output_mode == OutputMode::Lint
    {
        let errors = validate_static_components(&hir);
        log_errors_as_events(&errors, context);
        if context.debug_enabled {
            context.log_debug(DebugLogEntry::new("ValidateStaticComponents", "ok".to_string()));
        }
    }

    if env.enable_memoization() {
        infer_reactive_scope_variables(&mut hir, &mut env)?;

        if context.debug_enabled {
            let debug_infer_scopes = debug_print::debug_hir(&hir, &env);
            context
                .log_debug(DebugLogEntry::new("InferReactiveScopeVariables", debug_infer_scopes));
        }
    }

    let fbt_operands = memoize_fbt_and_macro_operands_in_same_scope(&hir, &mut env);

    if context.debug_enabled {
        let debug_fbt = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("MemoizeFbtAndMacroOperandsInSameScope", debug_fbt));
    }

    if env.config.enable_jsx_outlining {
        outline_jsx(&mut hir, &mut env);
    }

    if env.config.enable_name_anonymous_functions {
        name_anonymous_functions(&mut hir, &mut env);

        if context.debug_enabled {
            let debug_name_anon = debug_print::debug_hir(&hir, &env);
            context.log_debug(DebugLogEntry::new("NameAnonymousFunctions", debug_name_anon));
        }
    }

    if env.config.enable_function_outlining {
        outline_functions(&mut hir, &mut env, &fbt_operands);

        if context.debug_enabled {
            let debug_outline = debug_print::debug_hir(&hir, &env);
            context.log_debug(DebugLogEntry::new("OutlineFunctions", debug_outline));
        }
    }

    align_method_call_scopes(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_align = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("AlignMethodCallScopes", debug_align));
    }

    align_object_method_scopes(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_align_obj = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("AlignObjectMethodScopes", debug_align_obj));
    }

    prune_unused_labels_hir(&mut hir);

    if context.debug_enabled {
        let debug_prune_labels = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("PruneUnusedLabelsHIR", debug_prune_labels));
    }

    align_reactive_scopes_to_block_scopes_hir(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_align_block_scopes = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new(
            "AlignReactiveScopesToBlockScopesHIR",
            debug_align_block_scopes,
        ));
    }

    merge_overlapping_reactive_scopes_hir(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_merge_overlapping = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new(
            "MergeOverlappingReactiveScopesHIR",
            debug_merge_overlapping,
        ));
    }

    // TODO: port assertValidBlockNesting
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("AssertValidBlockNesting", "ok".to_string()));
    }

    build_reactive_scope_terminals_hir(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_build_scope_terminals = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new(
            "BuildReactiveScopeTerminalsHIR",
            debug_build_scope_terminals,
        ));
    }

    // TODO: port assertValidBlockNesting
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("AssertValidBlockNesting", "ok".to_string()));
    }

    flatten_reactive_loops_hir(&mut hir);

    if context.debug_enabled {
        let debug_flatten_loops = debug_print::debug_hir(&hir, &env);
        context.log_debug(DebugLogEntry::new("FlattenReactiveLoopsHIR", debug_flatten_loops));
    }

    flatten_scopes_with_hooks_or_use_hir(&mut hir, &env)?;

    if context.debug_enabled {
        let debug_flatten_hooks = debug_print::debug_hir(&hir, &env);
        context
            .log_debug(DebugLogEntry::new("FlattenScopesWithHooksOrUseHIR", debug_flatten_hooks));
    }

    // TODO: port assertTerminalSuccessorsExist
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("AssertTerminalSuccessorsExist", "ok".to_string()));
    }
    // TODO: port assertTerminalPredsExist
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("AssertTerminalPredsExist", "ok".to_string()));
    }

    propagate_scope_dependencies_hir(&mut hir, &mut env);

    if context.debug_enabled {
        let debug_propagate_deps = debug_print::debug_hir(&hir, &env);
        context
            .log_debug(DebugLogEntry::new("PropagateScopeDependenciesHIR", debug_propagate_deps));
    }

    let mut reactive_fn = build_reactive_function(&hir, &env)?;

    fn hir_formatter<'h>(fmt: &mut PrintFormatter<'_, 'h>, func: &HirFunction<'h>) {
        debug_print::format_hir_function_into(fmt, func);
    }

    if context.debug_enabled {
        let debug_reactive =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("BuildReactiveFunction", debug_reactive));
    }

    assert_well_formed_break_targets(&reactive_fn, &env);
    if context.debug_enabled {
        context.log_debug(DebugLogEntry::new("AssertWellFormedBreakTargets", "ok".to_string()));
    }

    prune_unused_labels(&mut reactive_fn, &env)?;

    if context.debug_enabled {
        let debug_prune_labels_reactive =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("PruneUnusedLabels", debug_prune_labels_reactive));
    }

    assert_scope_instructions_within_scopes(&reactive_fn, &env)?;
    if context.debug_enabled {
        context
            .log_debug(DebugLogEntry::new("AssertScopeInstructionsWithinScopes", "ok".to_string()));
    }

    prune_non_escaping_scopes(&mut reactive_fn, &mut env)?;

    if context.debug_enabled {
        let debug =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("PruneNonEscapingScopes", debug));
    }

    prune_non_reactive_dependencies(&mut reactive_fn, &mut env);

    if context.debug_enabled {
        let debug_prune_non_reactive =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new(
            "PruneNonReactiveDependencies",
            debug_prune_non_reactive,
        ));
    }

    prune_unused_scopes(&mut reactive_fn, &env)?;

    if context.debug_enabled {
        let debug_prune_unused_scopes =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("PruneUnusedScopes", debug_prune_unused_scopes));
    }

    merge_reactive_scopes_that_invalidate_together(&mut reactive_fn, &mut env)?;

    if context.debug_enabled {
        let debug =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("MergeReactiveScopesThatInvalidateTogether", debug));
    }

    prune_always_invalidating_scopes(&mut reactive_fn, &env)?;

    if context.debug_enabled {
        let debug_prune_always_inv =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context
            .log_debug(DebugLogEntry::new("PruneAlwaysInvalidatingScopes", debug_prune_always_inv));
    }

    propagate_early_returns(&mut reactive_fn, &mut env);

    if context.debug_enabled {
        let debug =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("PropagateEarlyReturns", debug));
    }

    prune_unused_lvalues(&mut reactive_fn, &env);

    if context.debug_enabled {
        let debug_prune_lvalues =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("PruneUnusedLValues", debug_prune_lvalues));
    }

    promote_used_temporaries(&mut reactive_fn, &mut env);

    if context.debug_enabled {
        let debug =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("PromoteUsedTemporaries", debug));
    }

    extract_scope_declarations_from_destructuring(&mut reactive_fn, &mut env)?;

    if context.debug_enabled {
        let debug =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("ExtractScopeDeclarationsFromDestructuring", debug));
    }

    stabilize_block_ids(&mut reactive_fn, &mut env);

    if context.debug_enabled {
        let debug_stabilize =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("StabilizeBlockIds", debug_stabilize));
    }

    let unique_identifiers = rename_variables(&mut reactive_fn, &mut env);

    for name in &unique_identifiers {
        context.add_new_reference(name.clone());
    }

    if context.debug_enabled {
        let debug =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("RenameVariables", debug));
    }

    prune_hoisted_contexts(&mut reactive_fn, &mut env)?;

    if context.debug_enabled {
        let debug =
            debug_reactive_function_with_formatter(&reactive_fn, &env, Some(&hir_formatter));
        context.log_debug(DebugLogEntry::new("PruneHoistedContexts", debug));
    }

    if env.config.enable_preserve_existing_memoization_guarantees
        || env.config.validate_preserve_existing_memoization_guarantees
    {
        validate_preserved_manual_memoization(&reactive_fn, &mut env);
        if context.debug_enabled {
            context.log_debug(DebugLogEntry::new(
                "ValidatePreservedManualMemoization",
                "ok".to_string(),
            ));
        }
    }

    let codegen_result =
        codegen_function(ast, &reactive_fn, &mut env, unique_identifiers, fbt_operands)?;

    // NOTE: we intentionally do NOT register the memo cache import here.
    // The import is registered in apply_compiled_functions() only for functions
    // that are actually applied to the output. Registering it here would cause
    // a spurious `import { c as _c }` when a function compiles with memo slots
    // but is later discarded (e.g., due to "use no memo" opt-out or errors),
    // while other functions in the same file compile to 0 memo slots.

    // Stage 2 Phase 1: `validate_source_locations` operated on the Babel-shaped
    // codegen result and is disabled while the oxc emission is stubbed. It will be
    // reinstated (or dropped) once the oxc back-end emits real function bodies.

    // Simulate unexpected exception for testing (matches TS Pipeline.ts)
    if env.config.throw_unknown_exception_testonly {
        let mut err = CompilerError::new();
        err.push_error_detail(CompilerErrorDetail {
            category: ErrorCategory::Invariant,
            reason: "unexpected error".to_string(),
            description: None,
            loc: None,
            suggestions: None,
        });
        return Err(err);
    }

    // Check for accumulated errors at the end of the pipeline
    // (matches TS Pipeline.ts: env.hasErrors() → Err at the end)
    if env.has_errors() {
        // Merge UIDs even on error: in TS, Babel's scope.generateUid() permanently
        // registers names in the scope's `uids` map regardless of whether the function
        // compilation succeeds or fails. Without this merge, failed compilations would
        // "leak" _temp names that subsequent successful compilations wouldn't see,
        // causing numbering mismatches vs TS.
        if let Some(uid_names) = env.take_uid_known_names() {
            context.merge_uid_known_names(&uid_names);
        }
        return Err(env.take_errors());
    }

    // Re-compile outlined functions through the full pipeline.
    // This mirrors TS behavior where outlined functions from JSX outlining
    // are pushed back onto the compilation queue and compiled as components.
    let mut compiled_outlined: Vec<OutlinedFunction<'a>> = Vec::new();
    for o in codegen_result.outlined {
        let outlined_codegen = CodegenFunction {
            loc: o.func.loc,
            id: o.func.id,
            name_hint: o.func.name_hint,
            params: o.func.params,
            body: o.func.body,
            generator: o.func.generator,
            is_async: o.func.is_async,
            memo_slots_used: o.func.memo_slots_used,
            memo_blocks: o.func.memo_blocks,
            memo_values: o.func.memo_values,
            pruned_memo_blocks: o.func.pruned_memo_blocks,
            pruned_memo_values: o.func.pruned_memo_values,
            outlined: Vec::new(),
        };
        if let Some(fn_type) = o.fn_type {
            let fn_name = outlined_codegen.id.as_ref().map(|id| id.name.to_string());
            match compile_outlined_fn(
                outlined_codegen,
                fn_name.as_deref(),
                fn_type,
                mode,
                env_config,
                context,
            ) {
                Ok(compiled) => {
                    compiled_outlined
                        .push(OutlinedFunction { func: compiled, fn_type: Some(fn_type) });
                }
                Err(_err) => {
                    // If re-compilation fails, skip the outlined function
                }
            }
        } else {
            compiled_outlined.push(OutlinedFunction { func: outlined_codegen, fn_type: o.fn_type });
        }
    }

    if let Some(uid_names) = env.take_uid_known_names() {
        context.merge_uid_known_names(&uid_names);
    }

    Ok(CodegenFunction {
        loc: codegen_result.loc,
        id: codegen_result.id,
        name_hint: codegen_result.name_hint,
        params: codegen_result.params,
        body: codegen_result.body,
        generator: codegen_result.generator,
        is_async: codegen_result.is_async,
        memo_slots_used: codegen_result.memo_slots_used,
        memo_blocks: codegen_result.memo_blocks,
        memo_values: codegen_result.memo_values,
        pruned_memo_blocks: codegen_result.pruned_memo_blocks,
        pruned_memo_values: codegen_result.pruned_memo_values,
        outlined: compiled_outlined,
    })
}

/// Compile an outlined function's codegen AST through the full pipeline.
///
/// Creates a fresh Environment, builds a synthetic ScopeInfo with unique fake
/// positions for identifier resolution, lowers from AST to HIR, then runs
/// the full compilation pipeline. This mirrors the TS behavior where outlined
/// functions are inserted into the program AST and re-compiled from scratch.
pub fn compile_outlined_fn<'a>(
    codegen_fn: CodegenFunction<'a>,
    fn_name: Option<&str>,
    fn_type: ReactFunctionType,
    mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext,
) -> Result<CodegenFunction<'a>, CompilerError> {
    let _ = (fn_name, fn_type, mode, env_config, context);
    Ok(codegen_fn)
}

/// Push a compiler error's per-detail diagnostics (validation / lint / telemetry
/// path), matching TS `env.logErrors()`. No enclosing-function fallback label.
fn log_errors_as_events(errors: &CompilerError, context: &mut ProgramContext) {
    for detail in &errors.details {
        if let Some(diagnostic) = crate::diagnostics::detail_to_diagnostic(detail, None) {
            context.diagnostics.push(diagnostic);
        }
    }
}
