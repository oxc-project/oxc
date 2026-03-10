use oxc_ast::AstBuilder;
use rustc_hash::FxHashSet;

use crate::{
    compiler_error::CompilerError,
    hir::{
        HIRFunction, IdentifierId, ReactFunctionType, ReactiveFunction,
        environment::{CompilerOutputMode, Environment},
    },
    inference::infer_mutation_aliasing_effects::InferOptions,
    inference::infer_mutation_aliasing_ranges::InferRangesOptions,
    reactive_scopes::codegen_reactive_function::{CodegenOptions, CodegenOutput, OutlinedOutput},
};

/// The result of running the compiler pipeline.
#[derive(Debug)]
pub enum CompilerPipelineValue {
    /// A compiled HIR function (intermediate stage).
    Hir { name: String, value: Box<HIRFunction> },
    /// A reactive function (intermediate stage).
    Reactive { name: String, value: ReactiveFunction },
    /// Analysis output ready for codegen.
    Analyzed { name: String, value: PipelineOutput },
    /// A debug string representation.
    Debug { name: String, value: String },
}

/// Output of the analysis pipeline, ready for codegen.
#[derive(Debug)]
pub struct PipelineOutput {
    /// The processed reactive function.
    pub reactive_function: ReactiveFunction,
    /// Unique identifiers from RenameVariables.
    pub unique_identifiers: FxHashSet<String>,
    /// FBT operand identifiers from MemoizeFbt.
    pub fbt_operands: FxHashSet<IdentifierId>,
    /// Pre-processed outlined functions (each went through the analysis sub-pipeline).
    pub outlined: Vec<OutlinedPipelineOutput>,
    /// Accumulated non-fatal validation errors (from `env.recordError()` equivalent).
    /// These allow the pipeline to continue through codegen, but are checked after
    /// codegen completes — if any are present, the error is returned to the caller.
    pub recorded_errors: Option<CompilerError>,
}

/// Analysis output for a single outlined function.
#[derive(Debug)]
pub struct OutlinedPipelineOutput {
    /// The processed reactive function.
    pub reactive_function: ReactiveFunction,
    /// Unique identifiers from RenameVariables.
    pub unique_identifiers: FxHashSet<String>,
    /// The function type of the outlined function.
    pub fn_type: Option<ReactFunctionType>,
}

/// Run the analysis pipeline on a function (everything except codegen).
///
/// This is the main entry point for analysis. It takes a lowered HIR function
/// and runs all analysis, optimization, and reactive scope passes in the correct
/// order, returning a `PipelineOutput` that can be passed to `run_codegen()`.
///
/// # Errors
/// Returns a `CompilerError` if any pass fails.
pub fn run_pipeline(
    func: &mut HIRFunction,
    env: &Environment,
) -> Result<PipelineOutput, CompilerError> {
    // =========================================================================
    // Phase 1: HIR-level passes
    // =========================================================================

    // 0a. Check for unreachable blocks containing hoisted function declarations.
    // Port of HIRBuilder.ts lines 358-371: before RPO cleanup, check if any block
    // not in the RPO set contains a FunctionExpression instruction. This catches
    // patterns like `return foo(); function foo() {}` where the function declaration
    // is in unreachable code but gets hoisted by JavaScript semantics.
    {
        let rpo_set: rustc_hash::FxHashSet<crate::hir::BlockId> =
            crate::hir::hir_builder::compute_rpo_order(func.body.entry, &func.body.blocks)
                .into_iter()
                .collect();
        for (block_id, block) in &func.body.blocks {
            if !rpo_set.contains(block_id)
                && block.instructions.iter().any(|instr| {
                    matches!(instr.value, crate::hir::InstructionValue::FunctionExpression(_))
                })
            {
                let loc =
                    block.instructions.first().map_or(block.terminal.loc(), |instr| instr.loc);
                return Err(CompilerError::todo(
                    "Support functions with unreachable code that may contain hoisted declarations",
                    None,
                    loc,
                ));
            }
        }
    }

    // 0b. HIR cleanup — the TS compiler runs these at the end of HIR building
    // (HIRBuilder.ts:392-398). We run them here at the start of the pipeline to
    // remove unreachable blocks left over from HIR construction (e.g. dead
    // do-while loops whose body unconditionally breaks, unreachable for-update
    // blocks, or try-catch blocks whose handler is unreachable).
    crate::hir::hir_builder::reverse_postorder_blocks(&mut func.body);
    crate::hir::hir_builder::remove_unreachable_for_updates(&mut func.body);
    crate::hir::hir_builder::remove_dead_do_while_statements(&mut func.body);
    crate::hir::hir_builder::remove_unnecessary_try_catch(&mut func.body);

    // 1. MarkInstructionIds — assign unique IDs to all instructions and terminals.
    // The TS compiler calls markInstructionIds at the end of HIR building (HIRBuilder.ts:397).
    // This is critical: without unique IDs, mutable range tracking is broken.
    crate::hir::hir_builder::mark_instruction_ids(&mut func.body);
    crate::hir::hir_builder::mark_predecessors(&mut func.body);

    // 2. PruneMaybeThrows
    crate::optimization::prune_maybe_throws::prune_maybe_throws(func);

    // 3. ValidateContextVariableLValues
    crate::validation::validate_context_variable_lvalues::validate_context_variable_lvalues(func)?;

    // 4. ValidateUseMemo — context variable reassignment errors are fatal (thrown
    //    via `.unwrap()` in TS), while void-memo errors are non-fatal (logged via
    //    `fn.env.logErrors`).
    {
        let (fatal_errors, void_memo_errors) =
            crate::validation::validate_use_memo::validate_use_memo(func);
        fatal_errors?;
        func.env.log_errors(void_memo_errors);
    }

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
    crate::type_inference::infer_types::infer_types(func)?;

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
    crate::inference::analyse_functions::analyse_functions(func)?;

    // 17. InferMutationAliasingEffects
    let infer_opts = InferOptions { is_function_expression: false };
    crate::inference::infer_mutation_aliasing_effects::infer_mutation_aliasing_effects(
        func,
        &infer_opts,
    )?;

    // 18. OptimizeForSSR (optional)
    if env.output_mode == CompilerOutputMode::Ssr {
        crate::optimization::optimize_for_ssr::optimize_for_ssr(func);
    }

    // 19. DeadCodeElimination
    crate::optimization::dead_code_elimination::dead_code_elimination(func);

    // 20. PruneMaybeThrows (second pass)
    crate::optimization::prune_maybe_throws::prune_maybe_throws(func);

    // 20b. Ensure blocks are in RPO order for InferMutationAliasingRanges.
    // The TS reference maintains RPO order by calling reversePostorderBlocks in
    // pruneMaybeThrows and constantPropagation. Without RPO order, the alias graph
    // index values computed in Phase 1 of InferMutationAliasingRanges are incorrect,
    // leading to inverted mutable ranges (start > end) for blocks processed out of order.
    crate::hir::hir_builder::reverse_postorder_blocks(&mut func.body);
    crate::hir::hir_builder::mark_instruction_ids(&mut func.body);
    crate::hir::hir_builder::mark_predecessors(&mut func.body);

    // 21. InferMutationAliasingRanges
    let range_opts = InferRangesOptions { is_function_expression: false };
    crate::inference::infer_mutation_aliasing_ranges::infer_mutation_aliasing_ranges(
        func,
        &range_opts,
    )?;

    // 22. ValidateLocalsNotReassignedAfterRender
    if env.enable_validations {
        crate::validation::validate_locals_not_reassigned_after_render::validate_locals_not_reassigned_after_render(func)?;
    }

    // 23. Validations (conditional on config)
    if env.enable_validations {
        if env.config.assert_valid_mutable_ranges {
            crate::hir::assert_valid_mutable_ranges::assert_valid_mutable_ranges(func)?;
        }

        if env.config.validate_ref_access_during_render {
            func.env.record_errors(
                crate::validation::validate_no_ref_access_in_render::validate_no_ref_access_in_render(
                    func,
                ),
            );
        }
        if env.config.validate_no_set_state_in_render {
            crate::validation::validate_no_set_state_in_render::validate_no_set_state_in_render(
                func,
            )?;
        }

        if env.config.validate_no_derived_computations_in_effects_exp
            && env.output_mode == CompilerOutputMode::Lint
        {
            func.env.log_errors(crate::validation::validate_no_derived_computations_in_effects_exp::validate_no_derived_computations_in_effects_exp(func).into_result());
        } else if env.config.validate_no_derived_computations_in_effects {
            crate::validation::validate_no_derived_computations_in_effects::validate_no_derived_computations_in_effects(func)?;
        }

        if env.config.validate_no_set_state_in_effects
            && env.output_mode == CompilerOutputMode::Lint
        {
            func.env.log_errors(crate::validation::validate_no_set_state_in_effects::validate_no_set_state_in_effects(func).into_result());
        }

        if env.config.validate_no_jsx_in_try_statements
            && env.output_mode == CompilerOutputMode::Lint
        {
            func.env.log_errors(crate::validation::validate_no_jsx_in_try_statement::validate_no_jsx_in_try_statement(func).into_result());
        }

        if env.config.validate_no_impure_functions_in_render {
            crate::validation::validate_no_impure_functions_in_render::validate_no_impure_functions_in_render(func)?;
        }

        if env.config.validate_no_freezing_known_mutable_functions {
            func.env.record_errors(
                crate::validation::validate_no_freezing_known_mutable_functions::validate_no_freezing_known_mutable_functions(func),
            );
        }
    }

    // 24. InferReactivePlaces
    crate::inference::infer_reactive_places::infer_reactive_places(func);

    // ValidateExhaustiveDependencies (optional, relies on reactivity inference)
    if env.enable_validations
        && (env.config.validate_exhaustive_memoization_dependencies
            || env.config.validate_exhaustive_effect_dependencies
                != crate::hir::environment::ExhaustiveEffectDepsMode::Off)
    {
        crate::validation::validate_exhaustive_dependencies::validate_exhaustive_dependencies(
            func,
        )?;
    }

    // 25. RewriteInstructionKindsBasedOnReassignment
    crate::ssa::rewrite_instruction_kinds::rewrite_instruction_kinds_based_on_reassignment(func)?;

    // ValidateStaticComponents (optional, lint-only)
    if env.enable_validations
        && env.config.validate_static_components
        && env.output_mode == CompilerOutputMode::Lint
    {
        func.env.log_errors(
            crate::validation::validate_static_components::validate_static_components(func)
                .into_result(),
        );
    }

    // =========================================================================
    // Phase 3: Reactive scope passes (HIR-level)
    // =========================================================================

    // 26. InferReactiveScopeVariables
    if env.enable_memoization {
        crate::reactive_scopes::infer_reactive_scope_variables::infer_reactive_scope_variables(
            func,
        )?;
    }

    // 27. MemoizeFbtAndMacroOperandsInSameScope
    let fbt_operands =
        crate::hir::memoize_fbt_operands::memoize_fbt_and_macro_operands_in_same_scope(func);

    // OutlineJSX (optional)
    if env.config.enable_jsx_outlining {
        crate::optimization::outline_jsx::outline_jsx(func);
    }

    // NameAnonymousFunctions (optional)
    if env.config.enable_name_anonymous_functions {
        crate::transform::name_anonymous_functions::name_anonymous_functions(func);
    }

    // OutlineFunctions (optional)
    if env.config.enable_function_outlining {
        crate::optimization::outline_functions::outline_functions(func, &fbt_operands);
    }

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
    crate::hir::assert_valid_block_nesting::assert_valid_block_nesting(func);

    // 33. BuildReactiveScopeTerminals
    crate::hir::build_reactive_scope_terminals_hir::build_reactive_scope_terminals_hir(func);

    // AssertValidBlockNesting (again)
    crate::hir::assert_valid_block_nesting::assert_valid_block_nesting(func);

    // 33b. Re-establish RPO order after BuildReactiveScopeTerminals splits blocks.
    // FlattenReactiveLoopsHIR iterates blocks in insertion order (matching the TS
    // reference which iterates fn.body.blocks in Map insertion order). After
    // BuildReactiveScopeTerminals creates new blocks, re-running reverse_postorder_blocks
    // ensures the loop terminals appear before their loop-body scope blocks, so that
    // active_loops is populated before we visit the scopes to prune.
    crate::hir::hir_builder::reverse_postorder_blocks(&mut func.body);
    crate::hir::hir_builder::mark_instruction_ids(&mut func.body);
    crate::hir::hir_builder::mark_predecessors(&mut func.body);

    // 34. FlattenReactiveLoops
    crate::reactive_scopes::flatten::flatten_reactive_loops_hir(func);

    // 35. FlattenScopesWithHooksOrUse
    crate::reactive_scopes::flatten::flatten_scopes_with_hooks_or_use_hir(func);

    // AssertTerminalSuccessorsExist + AssertTerminalPredsExist
    crate::hir::assertions::assert_terminal_successors_exist(func)?;
    crate::hir::assertions::assert_terminal_preds_exist(func)?;

    // 36. PropagateScopeDependencies
    crate::hir::propagate_scope_dependencies_hir::propagate_scope_dependencies_hir(func)?;

    // =========================================================================
    // Phase 4: Build reactive function (HIR -> Reactive tree)
    // =========================================================================

    // 37. BuildReactiveFunction
    let mut reactive_function =
        crate::reactive_scopes::build_reactive_function::build_reactive_function(func)?;

    // AssertWellFormedBreakTargets
    crate::reactive_scopes::assert_well_formed_break_targets::assert_well_formed_break_targets(
        &reactive_function,
    )?;

    // PruneUnusedLabels (reactive-level)
    crate::reactive_scopes::prune_unused_labels::prune_unused_labels(&mut reactive_function);

    // AssertScopeInstructionsWithinScopes
    crate::reactive_scopes::assert_scope_instructions_within_scopes::assert_scope_instructions_within_scopes(&reactive_function)?;

    // =========================================================================
    // Phase 5: Reactive function passes
    // =========================================================================

    // 38. PruneNonEscapingScopes
    let prune_opts = crate::reactive_scopes::prune_non_escaping_scopes::PruneOptions {
        memoize_jsx_elements: !env.config.enable_forest,
        force_memoize_primitives: env.config.enable_forest
            || env.config.enable_preserve_existing_memoization_guarantees,
        shapes: &env.shapes,
        env,
    };
    crate::reactive_scopes::prune_non_escaping_scopes::prune_non_escaping_scopes(
        &mut reactive_function,
        &prune_opts,
    );

    // 39. PruneNonReactiveDependencies
    crate::reactive_scopes::prune::prune_non_reactive_dependencies(&mut reactive_function);

    // 40. PruneUnusedScopes
    crate::reactive_scopes::prune::prune_unused_scopes(&mut reactive_function);

    // 41. MergeReactiveScopesThatInvalidateTogether
    crate::reactive_scopes::merge_scopes_that_invalidate_together::merge_reactive_scopes_that_invalidate_together(&mut reactive_function);

    // 42. PruneAlwaysInvalidatingScopes
    crate::reactive_scopes::prune::prune_always_invalidating_scopes(&mut reactive_function);

    // 43. PropagateEarlyReturns
    crate::reactive_scopes::propagate_early_returns::propagate_early_returns(
        &mut reactive_function,
        &mut func.env,
    );

    // PruneUnusedLValues
    crate::reactive_scopes::prune_unused_lvalues::prune_unused_lvalues(&mut reactive_function);

    // 44. PromoteUsedTemporaries
    crate::reactive_scopes::promote_used_temporaries::promote_used_temporaries(
        &mut reactive_function,
    );

    // 45. ExtractScopeDeclarationsFromDestructuring
    crate::reactive_scopes::extract_scope_declarations::extract_scope_declarations_from_destructuring(&mut reactive_function, &mut func.env);

    // 46. StabilizeBlockIds
    crate::reactive_scopes::stabilize_block_ids::stabilize_block_ids(&mut reactive_function);

    // 47. RenameVariables
    let unique_identifiers =
        crate::reactive_scopes::rename_variables::rename_variables(&mut reactive_function);

    // 48. PruneHoistedContexts
    crate::reactive_scopes::prune::prune_hoisted_contexts(&mut reactive_function)?;

    // 49. ValidatePreservedManualMemoization (optional)
    if env.config.enable_preserve_existing_memoization_guarantees
        || env.config.validate_preserve_existing_memoization_guarantees
    {
        func.env.record_errors(
            crate::validation::validate_preserved_manual_memoization::validate_preserved_manual_memoization(&reactive_function),
        );
    }

    // =========================================================================
    // Process outlined functions (analysis sub-pipeline only)
    // =========================================================================
    // Each outlined function goes through: build reactive function,
    // prune unused labels/lvalues, prune hoisted contexts, and rename variables.
    let mut outlined = Vec::new();
    for entry in func.env.get_outlined_functions() {
        let mut outlined_reactive =
            crate::reactive_scopes::build_reactive_function::build_reactive_function(&entry.func)?;
        crate::reactive_scopes::prune_unused_labels::prune_unused_labels(&mut outlined_reactive);
        crate::reactive_scopes::prune_unused_lvalues::prune_unused_lvalues(&mut outlined_reactive);
        crate::reactive_scopes::prune::prune_hoisted_contexts(&mut outlined_reactive)?;
        let outlined_identifiers =
            crate::reactive_scopes::rename_variables::rename_variables(&mut outlined_reactive);
        outlined.push(OutlinedPipelineOutput {
            reactive_function: outlined_reactive,
            unique_identifiers: outlined_identifiers,
            fn_type: entry.fn_type,
        });
    }

    let recorded_errors = func.env.take_recorded_errors();
    Ok(PipelineOutput {
        reactive_function,
        unique_identifiers,
        fbt_operands,
        outlined,
        recorded_errors,
    })
}

/// Run the codegen phase on the output of `run_pipeline()`.
///
/// Takes the analysis output, the environment, and an AST builder, and produces
/// the final `CodegenOutput` with all outlined functions attached.
///
/// # Errors
/// Returns a `CompilerError` if codegen or post-codegen validation fails.
pub fn run_codegen<'a>(
    pipeline_output: PipelineOutput,
    env: &Environment,
    ast: AstBuilder<'a>,
    cache_identifier_name: &str,
    original_func: Option<&crate::hir::build_hir::LowerableFunction<'_>>,
) -> Result<CodegenOutput<'a>, CompilerError> {
    let PipelineOutput {
        reactive_function,
        unique_identifiers,
        fbt_operands,
        outlined,
        recorded_errors,
    } = pipeline_output;

    // 50. CodegenFunction
    let fbt_operands_for_outlined = fbt_operands.clone();
    let enable_reset_cache = env.config.enable_reset_cache_on_source_file_changes == Some(true);
    let source_code = env.code.clone();
    let codegen_options = CodegenOptions {
        unique_identifiers,
        fbt_operands,
        enable_reset_cache_on_source_file_changes: enable_reset_cache,
        code: source_code,
        enable_emit_hook_guards: env.config.enable_emit_hook_guards.clone(),
        enable_emit_instrument_forget: env.config.enable_emit_instrument_forget.clone(),
        fn_id: reactive_function.id.clone(),
        filename: env.filename.clone(),
        output_mode: env.output_mode,
        shapes: env.shapes.clone(),
        enable_name_anonymous_functions: env.config.enable_name_anonymous_functions,
        cache_identifier_name: cache_identifier_name.to_string(),
    };
    let mut codegen_output = crate::reactive_scopes::codegen_reactive_function::codegen_function(
        &reactive_function,
        codegen_options,
        ast,
    )?;

    // 51. Codegen outlined functions
    let mut outlined_fns = Vec::new();
    for entry in outlined {
        let outlined_codegen_options = CodegenOptions {
            unique_identifiers: entry.unique_identifiers,
            fbt_operands: fbt_operands_for_outlined.clone(),
            enable_reset_cache_on_source_file_changes: false,
            code: None,
            enable_emit_hook_guards: env.config.enable_emit_hook_guards.clone(),
            enable_emit_instrument_forget: None,
            fn_id: None,
            filename: None,
            output_mode: env.output_mode,
            shapes: env.shapes.clone(),
            enable_name_anonymous_functions: env.config.enable_name_anonymous_functions,
            cache_identifier_name: cache_identifier_name.to_string(),
        };
        let outlined_ast = crate::reactive_scopes::codegen_reactive_function::codegen_function(
            &entry.reactive_function,
            outlined_codegen_options,
            ast,
        )?;
        outlined_fns.push(OutlinedOutput { fn_: outlined_ast, fn_type: entry.fn_type });
    }
    codegen_output.outlined = outlined_fns;

    // ValidateSourceLocations (optional)
    if env.config.validate_source_locations {
        crate::validation::validate_source_locations::validate_source_locations(
            &codegen_output,
            original_func,
        )?;
    }

    // [TESTING ONLY] Simulate an unexpected exception during compilation.
    if env.config.throw_unknown_exception_testonly {
        return Err(crate::compiler_error::CompilerError::invalid_config(
            "unexpected error",
            None,
            None,
        ));
    }

    // Check for accumulated non-fatal validation errors from the pipeline.
    // In the TS upstream, validation passes record errors via env.recordError()
    // and the pipeline continues through codegen. At the end of the pipeline
    // (Pipeline.ts:527), env.hasErrors() checks for accumulated errors and
    // returns them. This matches that pattern: codegen runs to completion,
    // but the error is still returned to the caller.
    if let Some(errors) = recorded_errors
        && errors.has_errors()
    {
        return Err(errors);
    }

    Ok(codegen_output)
}

/// Resolve the output mode for compilation.
pub fn resolve_output_mode(
    output_mode: Option<CompilerOutputMode>,
    no_emit: bool,
) -> CompilerOutputMode {
    if let Some(mode) = output_mode {
        return mode;
    }
    if no_emit { CompilerOutputMode::Lint } else { CompilerOutputMode::Client }
}
