
use crate::{
    compiler_error::CompilerError,
    hir::{
        HIRFunction, ReactiveFunction,
        environment::{CompilerOutputMode, Environment},
    },
    inference::infer_mutation_aliasing_effects::InferOptions,
    inference::infer_mutation_aliasing_ranges::InferRangesOptions,
    reactive_scopes::codegen_reactive_function::{CodegenFunction, CodegenOptions},
};

/// The result of running the compiler pipeline.
#[derive(Debug)]
pub enum CompilerPipelineValue {
    /// A compiled HIR function (intermediate stage).
    Hir { name: String, value: HIRFunction },
    /// A reactive function (intermediate stage).
    Reactive { name: String, value: ReactiveFunction },
    /// A compiled output function.
    Ast { name: String, value: CodegenFunction },
    /// A debug string representation.
    Debug { name: String, value: String },
}

/// Run the compiler pipeline on a function.
///
/// This is the main entry point for compilation. It takes a lowered HIR function
/// and runs all analysis, optimization, and codegen passes in the correct order.
///
/// # Errors
/// Returns a `CompilerError` if any pass fails.
pub fn run_pipeline(
    func: &mut HIRFunction,
    env: &Environment,
) -> Result<CodegenFunction, CompilerError> {
    // =========================================================================
    // Phase 1: HIR-level passes
    // =========================================================================

    // 2. PruneMaybeThrows
    crate::optimization::prune_maybe_throws::prune_maybe_throws(func);

    // 3. ValidateContextVariableLValues
    crate::validation::validate_context_variable_lvalues::validate_context_variable_lvalues(func)?;

    // 4. ValidateUseMemo
    crate::validation::validate_use_memo::validate_use_memo(func)?;

    // 5. DropManualMemoization (when memoization is enabled)
    if env.enable_drop_manual_memoization {
        crate::inference::drop_manual_memoization::drop_manual_memoization(func)?;
    }

    // 6. InlineImmediatelyInvokedFunctionExpressions
    crate::inference::inline_iife::inline_immediately_invoked_function_expressions(func);

    // 7. MergeConsecutiveBlocks
    crate::hir::merge_consecutive_blocks::merge_consecutive_blocks(func);

    // 8. AssertConsistentIdentifiers + AssertTerminalSuccessorsExist
    crate::hir::assertions::assert_consistent_identifiers(func)?;
    crate::hir::assertions::assert_terminal_successors_exist(func)?;

    // 9. EnterSSA
    crate::ssa::enter_ssa::enter_ssa(func, env)?;

    // 10. EliminateRedundantPhi
    crate::ssa::eliminate_redundant_phi::eliminate_redundant_phi(func, None);

    // AssertConsistentIdentifiers
    crate::hir::assertions::assert_consistent_identifiers(func)?;

    // 11. ConstantPropagation
    crate::optimization::constant_propagation::constant_propagation(func);

    // 12. InferTypes
    crate::type_inference::infer_types::infer_types(func);

    // =========================================================================
    // Phase 2: Validation + Analysis
    // =========================================================================

    // 13. ValidateHooksUsage (optional)
    if env.enable_validations && env.config.validate_hooks_usage {
        crate::validation::validate_hooks_usage::validate_hooks_usage(func)?;
    }

    // 14. ValidateNoCapitalizedCalls (optional)
    if env.enable_validations && env.config.validate_no_capitalized_calls.is_some() {
        crate::validation::validate_no_capitalized_calls::validate_no_capitalized_calls(func)?;
    }

    // 15. OptimizePropsMethodCalls
    crate::optimization::optimize_props_method_calls::optimize_props_method_calls(func);

    // 16. AnalyseFunctions
    crate::inference::analyse_functions::analyse_functions(func);

    // 17. InferMutationAliasingEffects
    let infer_opts = InferOptions { is_function_expression: false };
    crate::inference::infer_mutation_aliasing_effects::infer_mutation_aliasing_effects(
        func, &infer_opts,
    )?;

    // 18. OptimizeForSSR (optional)
    if env.output_mode == CompilerOutputMode::Ssr {
        crate::optimization::optimize_for_ssr::optimize_for_ssr(func);
    }

    // 19. DeadCodeElimination
    crate::optimization::dead_code_elimination::dead_code_elimination(func);

    // 20. PruneMaybeThrows (second pass)
    crate::optimization::prune_maybe_throws::prune_maybe_throws(func);

    // 21. InferMutationAliasingRanges
    let range_opts = InferRangesOptions { is_function_expression: false };
    crate::inference::infer_mutation_aliasing_ranges::infer_mutation_aliasing_ranges(
        func, range_opts,
    )?;

    // 22. ValidateLocalsNotReassignedAfterRender
    if env.enable_validations {
        crate::validation::validate_locals_not_reassigned_after_render::validate_locals_not_reassigned_after_render(func);
    }

    // 23. Validations (conditional on config)
    if env.enable_validations {
        if env.config.validate_ref_access_during_render {
            crate::validation::validate_no_ref_access_in_render::validate_no_ref_access_in_render(func)?;
        }
        if env.config.validate_no_set_state_in_render {
            crate::validation::validate_no_set_state_in_render::validate_no_set_state_in_render(func)?;
        }
        crate::validation::validate_no_freezing_known_mutable_functions::validate_no_freezing_known_mutable_functions(func)?;
    }

    // 24. InferReactivePlaces
    crate::inference::infer_reactive_places::infer_reactive_places(func);

    // 25. RewriteInstructionKindsBasedOnReassignment
    crate::ssa::rewrite_instruction_kinds::rewrite_instruction_kinds_based_on_reassignment(func)?;

    // =========================================================================
    // Phase 3: Reactive scope passes (HIR-level)
    // =========================================================================

    // 26. InferReactiveScopeVariables
    if env.enable_memoization {
        crate::reactive_scopes::infer_reactive_scope_variables::infer_reactive_scope_variables(func);
    }

    // 27. MemoizeFbtAndMacroOperandsInSameScope
    let fbt_operands = crate::hir::memoize_fbt_operands::memoize_fbt_and_macro_operands_in_same_scope(func);

    // 28. AlignMethodCallScopes
    crate::reactive_scopes::align_scopes::align_method_call_scopes(func);

    // 29. AlignObjectMethodScopes
    crate::reactive_scopes::align_scopes::align_object_method_scopes(func);

    // 30. PruneUnusedLabelsHIR
    crate::hir::prune_unused_labels_hir::prune_unused_labels_hir(func);

    // 31. AlignReactiveScopesToBlockScopes
    crate::reactive_scopes::align_scopes::align_reactive_scopes_to_block_scopes_hir(func);

    // 32. MergeOverlappingReactiveScopes
    crate::hir::merge_overlapping_reactive_scopes_hir::merge_overlapping_reactive_scopes_hir(func);

    // AssertValidBlockNesting
    crate::hir::assert_valid_block_nesting::assert_valid_block_nesting(func)?;

    // 33. BuildReactiveScopeTerminals
    crate::hir::build_reactive_scope_terminals_hir::build_reactive_scope_terminals_hir(func);

    // AssertValidBlockNesting (again)
    crate::hir::assert_valid_block_nesting::assert_valid_block_nesting(func)?;

    // 34. FlattenReactiveLoops
    crate::reactive_scopes::flatten::flatten_reactive_loops_hir(func);

    // 35. FlattenScopesWithHooksOrUse
    crate::reactive_scopes::flatten::flatten_scopes_with_hooks_or_use_hir(func);

    // AssertTerminalSuccessorsExist + AssertTerminalPredsExist
    crate::hir::assertions::assert_terminal_successors_exist(func)?;
    crate::hir::assertions::assert_terminal_preds_exist(func)?;

    // 36. PropagateScopeDependencies
    crate::hir::propagate_scope_dependencies_hir::propagate_scope_dependencies_hir(func);

    // =========================================================================
    // Phase 4: Build reactive function (HIR → Reactive tree)
    // =========================================================================

    // 37. BuildReactiveFunction
    let mut reactive_function = crate::reactive_scopes::build_reactive_function::build_reactive_function(func);

    // =========================================================================
    // Phase 5: Reactive function passes
    // =========================================================================

    // 38. PruneNonEscapingScopes
    crate::reactive_scopes::prune_non_escaping_scopes::prune_non_escaping_scopes(&mut reactive_function);

    // 39. PruneNonReactiveDependencies
    crate::reactive_scopes::prune::prune_non_reactive_dependencies(&mut reactive_function);

    // 40. PruneUnusedScopes
    crate::reactive_scopes::prune::prune_unused_scopes(&mut reactive_function);

    // 41. MergeReactiveScopesThatInvalidateTogether
    crate::reactive_scopes::merge_scopes_that_invalidate_together::merge_reactive_scopes_that_invalidate_together(&mut reactive_function);

    // 42. PruneAlwaysInvalidatingScopes
    crate::reactive_scopes::prune::prune_always_invalidating_scopes(&mut reactive_function);

    // 43. PropagateEarlyReturns
    crate::reactive_scopes::propagate_early_returns::propagate_early_returns(&mut reactive_function);

    // 44. PromoteUsedTemporaries
    crate::reactive_scopes::promote_used_temporaries::promote_used_temporaries(&mut reactive_function);

    // 45. ExtractScopeDeclarationsFromDestructuring
    crate::reactive_scopes::extract_scope_declarations::extract_scope_declarations_from_destructuring(&mut reactive_function);

    // 46. StabilizeBlockIds
    crate::reactive_scopes::stabilize_block_ids::stabilize_block_ids(&mut reactive_function);

    // 47. RenameVariables
    let unique_identifiers = crate::reactive_scopes::rename_variables::rename_variables(&reactive_function);

    // 48. PruneHoistedContexts
    crate::reactive_scopes::prune::prune_hoisted_contexts(&mut reactive_function);

    // 49. ValidatePreservedManualMemoization (optional)
    if env.config.enable_preserve_existing_memoization_guarantees
        || env.config.validate_preserve_existing_memoization_guarantees
    {
        crate::validation::validate_preserved_manual_memoization::validate_preserved_manual_memoization(&reactive_function)?;
    }

    // =========================================================================
    // Phase 6: Codegen
    // =========================================================================

    // 50. CodegenFunction
    let codegen_options = CodegenOptions {
        unique_identifiers,
        fbt_operands,
    };
    let ast = crate::reactive_scopes::codegen_reactive_function::codegen_function(
        &reactive_function,
        codegen_options,
    )?;

    Ok(ast)
}

/// Resolve the output mode for compilation.
pub fn resolve_output_mode(
    output_mode: Option<CompilerOutputMode>,
    no_emit: bool,
) -> CompilerOutputMode {
    if let Some(mode) = output_mode {
        return mode;
    }
    if no_emit {
        CompilerOutputMode::Lint
    } else {
        CompilerOutputMode::Client
    }
}
