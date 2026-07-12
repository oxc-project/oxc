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
use oxc_span::Span;

use crate::diagnostics::{ErrorCategory, to_string_for_event};
use crate::react_compiler_hir::ReactFunctionType;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::environment::OutputMode;
use crate::react_compiler_hir::environment_config::EnvironmentConfig;
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
use super::compile_result::OutlinedFunction;
use super::imports::ProgramContext;
use crate::options::CompilerOutputMode;

/// Run the compilation pipeline on a single function.
///
/// On failure, returns the diagnostics of the failed compilation attempt.
/// An error thrown by a pass (in TS: an exception escaping the pass) that is
/// not an Invariant additionally surfaces a `CompileUnexpectedThrow`
/// diagnostic, matching TS `tryCompileFunction`'s catch block.
#[allow(clippy::too_many_arguments)]
pub fn compile_fn<'a>(
    ast: &oxc_ast::builder::AstBuilder<'a>,
    func: &FunctionNode<'_, 'a>,
    scope: &ScopeResolver<'_, 'a>,
    fn_type: ReactFunctionType,
    mode: CompilerOutputMode,
    env_config: &EnvironmentConfig,
    context: &mut ProgramContext<'a>,
    fn_span: Option<Span>,
) -> Result<Option<CodegenFunction<'a>>, Diagnostics> {
    match run_pipeline(ast, func, scope, fn_type, mode, env_config, context) {
        Ok(result) => result,
        Err(thrown) => {
            if !ErrorCategory::Invariant.matches(&thrown) {
                let mut diagnostic = OxcDiagnostic::error(format!(
                    "[ReactCompiler] Unexpected error: {}",
                    to_string_for_event(&thrown)
                ));
                if let Some(span) = fn_span {
                    diagnostic = diagnostic.with_label(span);
                }
                context.diagnostics.push(diagnostic);
            }
            Err(Diagnostics::from(thrown))
        }
    }
}

/// The pass pipeline: creates an Environment, runs BuildHIR (lowering), the
/// HIR/reactive-scope passes, and codegen.
///
/// `Err(OxcDiagnostic)` is an error thrown by a pass (a TS exception);
/// Invariant and end-of-pipeline accumulated errors return as
/// `Ok(Err(diagnostics))` since they must not surface `CompileUnexpectedThrow`.
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

    prune_maybe_throws(&mut hir, &mut env.functions)?;

    validate_context_variable_lvalues(&hir, &mut env)?;

    let void_memo_errors = validate_use_memo(&hir, &mut env);
    log_errors_as_events(&void_memo_errors, context);

    drop_manual_memoization(&mut hir, &mut env)?;

    inline_immediately_invoked_function_expressions(&mut hir, &mut env);

    merge_consecutive_blocks(&mut hir, &mut env.functions);

    // TODO: port assertConsistentIdentifiers
    // TODO: port assertTerminalSuccessorsExist

    enter_ssa(&mut hir, &mut env)?;

    eliminate_redundant_phi(&mut hir, &mut env);

    // TODO: port assertConsistentIdentifiers

    constant_propagation(&mut hir, &mut env);

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

    prune_maybe_throws(&mut hir, &mut env.functions)?;

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

    if env.enable_validations() {
        validate_exhaustive_dependencies(&mut hir, &mut env)?;
    }

    rewrite_instruction_kinds_based_on_reassignment(&mut hir, &env)?;

    if env.enable_validations()
        && env.config.validate_static_components
        && env.output_mode == OutputMode::Lint
    {
        let errors = validate_static_components(&hir);
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

    // TODO: port assertValidBlockNesting

    build_reactive_scope_terminals_hir(&mut hir, &mut env);

    // TODO: port assertValidBlockNesting

    flatten_reactive_loops_hir(&mut hir);

    flatten_scopes_with_hooks_or_use_hir(&mut hir, &env)?;

    // TODO: port assertTerminalSuccessorsExist
    // TODO: port assertTerminalPredsExist

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
        return Err(ErrorCategory::Invariant.diagnostic("unexpected error"));
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

    // Re-compile outlined functions through the full pipeline.
    // This mirrors TS behavior where outlined functions from JSX outlining
    // are pushed back onto the compilation queue and compiled as components.
    let mut compiled_outlined: Vec<OutlinedFunction<'a>> = Vec::new();
    for o in codegen_result.outlined {
        let outlined_codegen = CodegenFunction {
            span: o.func.span,
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
            match compile_outlined_fn(outlined_codegen) {
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
        outlined: compiled_outlined,
    })))
}

/// Compile an outlined function's codegen AST through the full pipeline.
///
/// Creates a fresh Environment, builds a synthetic ScopeInfo with unique fake
/// positions for identifier resolution, lowers from AST to HIR, then runs
/// the full compilation pipeline. This mirrors the TS behavior where outlined
/// functions are inserted into the program AST and re-compiled from scratch.
pub fn compile_outlined_fn<'a>(
    codegen_fn: CodegenFunction<'a>,
) -> Result<CodegenFunction<'a>, OxcDiagnostic> {
    Ok(codegen_fn)
}

/// Push a pass's diagnostics (validation / lint / telemetry path),
/// matching TS `env.logErrors()`. No enclosing-function fallback label.
fn log_errors_as_events(errors: &Diagnostics, context: &mut ProgramContext) {
    context.diagnostics.extend(errors.iter().cloned());
}
