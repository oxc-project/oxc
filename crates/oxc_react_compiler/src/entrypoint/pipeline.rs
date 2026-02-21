/// The React Compiler pipeline — orchestrates all compilation passes.
///
/// Port of `Entrypoint/Pipeline.ts` from the React Compiler.
///
/// This module defines the compilation pipeline that transforms a function's
/// HIR through multiple optimization and analysis passes to produce optimized
/// output code.
///
/// The pipeline order:
/// 1. Lower AST → HIR
/// 2. PruneMaybeThrows
/// 3. ValidateContextVariableLValues
/// 4. ValidateUseMemo
/// 5. DropManualMemoization (optional)
/// 6. InlineIIFEs
/// 7. MergeConsecutiveBlocks
/// 8. EnterSSA
/// 9. EliminateRedundantPhi
/// 10. ConstantPropagation
/// 11. InferTypes
/// 12. ValidateHooksUsage (optional)
/// 13. ValidateNoCapitalizedCalls (optional)
/// 14. OptimizePropsMethodCalls
/// 15. AnalyseFunctions
/// 16. InferMutationAliasingEffects
/// 17. OptimizeForSSR (optional)
/// 18. DeadCodeElimination
/// 19. PruneMaybeThrows
/// 20. InferMutationAliasingRanges
/// 21. Validations (refs, setState, effects, etc.)
/// 22. InferReactivePlaces
/// 23. RewriteInstructionKindsBasedOnReassignment
/// 24. InferReactiveScopeVariables
/// 25. Various reactive scope passes
/// 26. BuildReactiveFunction
/// 27. Reactive function passes
/// 28. CodegenFunction
use crate::{
    compiler_error::CompilerError,
    hir::{
        HIRFunction,
        environment::{CompilerOutputMode, Environment},
    },
};

/// The result of running the compiler pipeline.
#[derive(Debug)]
pub enum CompilerPipelineValue {
    /// A compiled HIR function (intermediate stage).
    Hir { name: String, value: HIRFunction },
    /// A debug string representation.
    Debug { name: String, value: String },
}

/// Run the compiler pipeline on a function.
///
/// This is the main entry point for compilation. It takes a lowered HIR function
/// and runs all analysis, optimization, and codegen passes.
///
/// # Errors
/// Returns a `CompilerError` if any pass fails.
pub fn run_pipeline(
    func: &mut HIRFunction,
    env: &mut Environment,
) -> Result<(), CompilerError> {
    // Phase 1: HIR-level optimizations and analysis

    // PruneMaybeThrows
    crate::optimization::prune_maybe_throws::prune_maybe_throws(func);

    // ValidateContextVariableLValues
    crate::validation::validate_context_variable_lvalues::validate_context_variable_lvalues(func)?;

    // MergeConsecutiveBlocks
    crate::hir::merge_consecutive_blocks::merge_consecutive_blocks(func);

    // AssertConsistentIdentifiers
    crate::hir::assertions::assert_consistent_identifiers(func)?;
    crate::hir::assertions::assert_terminal_successors_exist(func)?;

    // EnterSSA
    crate::ssa::enter_ssa::enter_ssa(func, env)?;

    // EliminateRedundantPhi
    crate::ssa::eliminate_redundant_phi::eliminate_redundant_phi(func, None);

    // AssertConsistentIdentifiers
    crate::hir::assertions::assert_consistent_identifiers(func)?;

    // ConstantPropagation
    crate::optimization::constant_propagation::constant_propagation(func);

    // InferTypes
    crate::type_inference::infer_types::infer_types(func);

    // OptimizePropsMethodCalls
    crate::optimization::optimize_props_method_calls::optimize_props_method_calls(func);

    // Phase 2: More analysis would go here
    // - InferMutationAliasingEffects
    // - OptimizeForSSR (if mode == SSR)
    // - DeadCodeElimination
    // - InferMutationAliasingRanges
    // - Various validations

    // DeadCodeElimination
    crate::optimization::dead_code_elimination::dead_code_elimination(func);

    // PruneMaybeThrows (second pass)
    crate::optimization::prune_maybe_throws::prune_maybe_throws(func);

    // RewriteInstructionKindsBasedOnReassignment
    crate::ssa::rewrite_instruction_kinds::rewrite_instruction_kinds_based_on_reassignment(func)?;

    // PruneUnusedLabelsHIR
    crate::hir::prune_unused_labels_hir::prune_unused_labels_hir(func);

    // Phase 3: Reactive scope passes would go here
    // - InferReactiveScopeVariables
    // - MemoizeFbtAndMacroOperands
    // - AlignMethodCallScopes
    // - AlignObjectMethodScopes
    // - AlignReactiveScopesToBlockScopes
    // - MergeOverlappingReactiveScopes
    // - BuildReactiveScopeTerminals
    // - FlattenReactiveLoops
    // - FlattenScopesWithHooksOrUse
    // - PropagateScopeDependencies

    // Phase 4: Build reactive function (HIR → Reactive tree)
    // - BuildReactiveFunction

    // Phase 5: Reactive function passes
    // - PruneUnusedLabels
    // - PruneNonEscapingScopes
    // - PruneNonReactiveDependencies
    // - PruneUnusedScopes
    // - MergeReactiveScopesThatInvalidateTogether
    // - PruneAlwaysInvalidatingScopes
    // - PropagateEarlyReturns
    // - PruneUnusedLValues
    // - PromoteUsedTemporaries
    // - ExtractScopeDeclarationsFromDestructuring
    // - StabilizeBlockIds
    // - RenameVariables
    // - PruneHoistedContexts
    // - ValidatePreservedManualMemoization

    // Phase 6: Codegen
    // - CodegenReactiveFunction

    Ok(())
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
