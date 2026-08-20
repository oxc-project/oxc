// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Compilation pipeline for a single function.
//!
//! Analogous to TS `Pipeline.ts` (`compileFn` → `run` → `runWithEnvironment`).
//! Currently runs BuildHIR (lowering) and PruneMaybeThrows.

use oxc_allocator::GetAllocator;
use oxc_diagnostics::{Diagnostics, OxcDiagnostic};

use crate::diagnostics;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::environment::OutputMode;
use crate::react_compiler_hir::environment_config::{EnvironmentConfig, ExhaustiveEffectDepsMode};
use crate::react_compiler_hir::{
    ReactFunctionType, assert_consistent_identifiers, assert_terminal_preds_exist,
    assert_terminal_successors_exist, assert_valid_block_nesting,
};
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
use super::imports::ProgramContext;
use crate::options::CompilerOutputMode;

/// Run the compilation pipeline on a single function.
///
/// On failure, returns the diagnostics of the failed compilation attempt.
#[allow(clippy::too_many_arguments)]
pub fn compile_fn<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    func: &FunctionNode<'_, 'a>,
    scope: &ScopeResolver<'_, 'a>,
    fn_type: ReactFunctionType,
    mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext<'a>,
) -> Result<Option<CodegenFunction<'a>>, Diagnostics> {
    match run_pipeline(ast, func, scope, fn_type, mode, env_config, context) {
        Ok(result) => result,
        Err(diagnostic) => Err(Diagnostics::from(diagnostic)),
    }
}

/// The pass pipeline: creates an Environment, runs BuildHIR (lowering), the
/// HIR/reactive-scope passes, and codegen.
///
/// `Err(OxcDiagnostic)` is a diagnostic that immediately bails out of a pass.
/// Invariant and end-of-pipeline accumulated errors return as
/// `Ok(Err(diagnostics))`.
#[allow(clippy::too_many_arguments)]
fn run_pipeline<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    func: &FunctionNode<'_, 'a>,
    scope: &ScopeResolver<'_, 'a>,
    fn_type: ReactFunctionType,
    mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext<'a>,
) -> Result<Result<Option<CodegenFunction<'a>>, Diagnostics>, OxcDiagnostic> {
    let mut env = Environment::with_config(ast.allocator(), env_config.clone());
    env.fn_type = fn_type;
    env.output_mode = match mode {
        CompilerOutputMode::Ssr => OutputMode::Ssr,
        CompilerOutputMode::Client => OutputMode::Client,
        CompilerOutputMode::Lint => OutputMode::Lint,
    };
    env.instrument_fn_name = context.instrument_fn_name;
    env.instrument_gating_name = context.instrument_gating_name;
    env.hook_guard_name = context.hook_guard_name;
    env.memo_cache_name = context.memo_cache_name;
    env.seed_uid_known_names(context.known_referenced_names());

    let mut hir = lower(func, scope, &mut env)?;

    // Check for Invariant errors after lowering, before logging HIR.
    // In TS, Invariant errors throw from recordError(), aborting lower() before
    // the HIR entry is logged. The thrown error contains ONLY the Invariant error,
    // not other recorded (non-Invariant) errors.
    if env.has_invariant_errors() {
        return Ok(Err(env.take_invariant_errors()));
    }

    // Lowering flags this when the function uses `using`/`await using`, whose disposal
    // semantics aren't preserved yet. Skip compiling it silently — no diagnostic — so
    // other functions in the file still compile.
    if env.skip_compilation {
        return Ok(Ok(None));
    }

    prune_maybe_throws(&mut hir, &mut env.functions, &env.identifiers, env.allocator)?;

    validate_context_variable_lvalues(&hir, &mut env)?;

    let void_memo_errors = validate_use_memo(&hir, &mut env);
    log_errors_as_events(&void_memo_errors, context);

    drop_manual_memoization(&mut hir, &mut env)?;

    inline_immediately_invoked_function_expressions(&mut hir, &mut env);

    merge_consecutive_blocks(&mut hir, &mut env.functions, env.allocator);

    assert_consistent_identifiers(&hir, &env.identifiers)?;
    assert_terminal_successors_exist(&hir)?;

    enter_ssa(&mut hir, &mut env)?;

    eliminate_redundant_phi(&mut hir, &mut env);

    assert_consistent_identifiers(&hir, &env.identifiers)?;

    constant_propagation(&mut hir, &mut env)?;

    infer_types(&mut hir, &mut env)?;

    if env.enable_validations() {
        if env.config.validate_hooks_usage {
            validate_hooks_usage(&hir, &mut env)?;
        }

        if env.config.validate_no_capitalized_calls.is_some() {
            validate_no_capitalized_calls(&hir, &mut env)?;
        }
    }

    optimize_props_method_calls(&mut hir, &env);

    analyse_functions(&mut hir, &mut env, &mut |_inner_func, _inner_env| {})?;

    if env.has_invariant_errors() {
        return Ok(Err(env.take_invariant_errors()));
    }

    infer_mutation_aliasing_effects(&mut hir, &mut env, false)?;

    if env.output_mode == OutputMode::Ssr {
        optimize_for_ssr(&mut hir, &env);
    }

    dead_code_elimination(&mut hir, &env);

    prune_maybe_throws(&mut hir, &mut env.functions, &env.identifiers, env.allocator)?;

    infer_mutation_aliasing_ranges(&mut hir, &mut env, false)?;

    if env.enable_validations() {
        validate_locals_not_reassigned_after_render(&hir, &mut env);

        if env.config.validate_ref_access_during_render {
            validate_no_ref_access_in_render(&hir, &mut env);
        }

        if env.config.validate_no_set_state_in_render {
            validate_no_set_state_in_render(&hir, &mut env)?;
        }

        if env.config.validate_no_derived_computations_in_effects_exp
            && env.output_mode == OutputMode::Lint
        {
            let errors = validate_no_derived_computations_in_effects_exp(&hir, &env)?;
            log_errors_as_events(&errors, context);
        } else if env.config.validate_no_derived_computations_in_effects {
            validate_no_derived_computations_in_effects(&hir, &mut env)?;
        }

        if env.config.validate_no_set_state_in_effects && env.output_mode == OutputMode::Lint {
            let errors = validate_no_set_state_in_effects(&hir, &env)?;
            log_errors_as_events(&errors, context);
        }

        if env.config.validate_no_jsx_in_try_statements && env.output_mode == OutputMode::Lint {
            let errors = validate_no_jsx_in_try_statement(&hir);
            log_errors_as_events(&errors, context);
        }

        validate_no_freezing_known_mutable_functions(&hir, &mut env);
    }

    infer_reactive_places(&mut hir, &mut env)?;

    if env.enable_validations()
        && (env.config.validate_exhaustive_memoization_dependencies
            || env.config.validate_exhaustive_effect_dependencies != ExhaustiveEffectDepsMode::Off)
    {
        validate_exhaustive_dependencies(&mut hir, &mut env)?;
    }

    rewrite_instruction_kinds_based_on_reassignment(&mut hir, &env)?;

    if env.enable_validations()
        && env.config.validate_static_components
        && env.output_mode == OutputMode::Lint
    {
        let errors = validate_static_components(&hir, &env.functions);
        log_errors_as_events(&errors, context);
    }

    if env.enable_memoization() {
        infer_reactive_scope_variables(&mut hir, &mut env)?;
    }

    let fbt_operands = memoize_fbt_and_macro_operands_in_same_scope(&hir, &mut env);

    if env.config.enable_jsx_outlining {
        outline_jsx(&mut hir, &mut env);
    }

    if env.config.enable_name_anonymous_functions {
        name_anonymous_functions(&mut hir, &mut env);
    }

    if env.config.enable_function_outlining {
        outline_functions(&mut hir, &mut env, &fbt_operands);
    }

    align_method_call_scopes(&mut hir, &mut env);

    align_object_method_scopes(&mut hir, &mut env);

    prune_unused_labels_hir(&mut hir);

    align_reactive_scopes_to_block_scopes_hir(&mut hir, &mut env);

    merge_overlapping_reactive_scopes_hir(&mut hir, &mut env);

    assert_valid_block_nesting(&hir, &env)?;

    build_reactive_scope_terminals_hir(&mut hir, &mut env)?;

    assert_valid_block_nesting(&hir, &env)?;

    flatten_reactive_loops_hir(&mut hir);

    flatten_scopes_with_hooks_or_use_hir(&mut hir, &env)?;

    assert_terminal_successors_exist(&hir)?;
    assert_terminal_preds_exist(&hir)?;

    propagate_scope_dependencies_hir(&mut hir, &mut env);

    let mut reactive_fn = build_reactive_function(&hir, &env)?;

    assert_well_formed_break_targets(&reactive_fn, &env);

    prune_unused_labels(&mut reactive_fn, &env)?;

    assert_scope_instructions_within_scopes(&reactive_fn, &env)?;

    prune_non_escaping_scopes(&mut reactive_fn, &mut env)?;

    prune_non_reactive_dependencies(&mut reactive_fn, &mut env);

    prune_unused_scopes(&mut reactive_fn, &env)?;

    merge_reactive_scopes_that_invalidate_together(&mut reactive_fn, &mut env)?;

    prune_always_invalidating_scopes(&mut reactive_fn, &env)?;

    propagate_early_returns(&mut reactive_fn, &mut env);

    prune_unused_lvalues(&mut reactive_fn, &env);

    promote_used_temporaries(&mut reactive_fn, &mut env);

    extract_scope_declarations_from_destructuring(&mut reactive_fn, &mut env)?;

    stabilize_block_ids(&mut reactive_fn, &mut env);

    let unique_identifiers = rename_variables(&mut reactive_fn, &mut env);

    for name in &unique_identifiers {
        context.add_new_reference(*name);
    }

    prune_hoisted_contexts(&mut reactive_fn, &env)?;

    if env.config.enable_preserve_existing_memoization_guarantees
        || env.config.validate_preserve_existing_memoization_guarantees
    {
        validate_preserved_manual_memoization(&reactive_fn, &mut env);
    }

    let codegen_result =
        codegen_function(ast, &reactive_fn, &mut env, unique_identifiers, fbt_operands)?;

    // NOTE: we intentionally do NOT register the memo cache import here.
    // The local name is reserved up front by `ProgramContext::reserve_memo_cache_name`,
    // and the import itself is registered in `ox_transform_program` only when an applied
    // function uses memo slots. Registering it here would cause a spurious
    // `import { c as _c }` when a function compiles with memo slots but is later
    // discarded (e.g., due to "use no memo" opt-out or errors), while other functions
    // in the same file compile to 0 memo slots.

    // Stage 2 Phase 1: `validate_source_locations` operated on the Babel-shaped
    // codegen result and is disabled while the oxc emission is stubbed. It will be
    // reinstated (or dropped) once the oxc back-end emits real function bodies.

    // Simulate unexpected exception for testing (matches TS Pipeline.ts)
    if env.config.throw_unknown_exception_testonly {
        return Err(diagnostics::invariant_unexpected_error());
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
        return Ok(Err(env.take_errors()));
    }

    if let Some(uid_names) = env.take_uid_known_names() {
        context.merge_uid_known_names(&uid_names);
    }

    Ok(Ok(Some(CodegenFunction {
        span: codegen_result.span,
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
        outlined: codegen_result.outlined,
    })))
}

/// Push a pass's diagnostics (validation / lint / telemetry path),
/// matching TS `env.logErrors()`. No enclosing-function fallback label.
fn log_errors_as_events(errors: &Diagnostics, context: &mut ProgramContext) {
    context.diagnostics.extend(errors.iter().cloned());
}
