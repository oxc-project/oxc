/// Analyse nested function expressions for mutation and aliasing effects.
///
/// Port of `Inference/AnalyseFunctions.ts` from the React Compiler.
///
/// For each nested function expression or object method, this pass:
/// 1. Recursively analyses inner functions
/// 2. Infers mutation/aliasing effects
/// 3. Runs DCE and reactive scope inference
/// 4. Populates context variable effects for the outer function
use crate::compiler_error::CompilerError;
use crate::hir::{
    Effect, HIRFunction, IdentifierId, InstructionId, InstructionValue, MutableRange,
};
use crate::inference::aliasing_effects::AliasingEffect;
use crate::inference::infer_mutation_aliasing_effects::{
    InferOptions, infer_mutation_aliasing_effects,
};
use crate::inference::infer_mutation_aliasing_ranges::{
    InferRangesOptions, infer_mutation_aliasing_ranges,
};
use crate::optimization::dead_code_elimination::dead_code_elimination;
use crate::reactive_scopes::infer_reactive_scope_variables::infer_reactive_scope_variables;
use crate::ssa::rewrite_instruction_kinds::rewrite_instruction_kinds_based_on_reassignment;
use rustc_hash::FxHashSet;

/// Analyse all nested function expressions in the given function.
///
/// # Errors
/// Returns a `CompilerError` if mutation aliasing inference fails for any nested function.
pub fn analyse_functions(func: &mut HIRFunction) -> Result<(), CompilerError> {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };

        for instr in &mut block.instructions {
            match &mut instr.value {
                InstructionValue::FunctionExpression(v) => {
                    lower_with_mutation_aliasing(&mut v.lowered_func.func)?;

                    // Reset mutable range for outer inference
                    for operand in &mut v.lowered_func.func.context {
                        operand.identifier.mutable_range =
                            MutableRange { start: InstructionId(0), end: InstructionId(0) };
                        operand.identifier.scope = None;
                    }
                }
                InstructionValue::ObjectMethod(v) => {
                    lower_with_mutation_aliasing(&mut v.lowered_func.func)?;

                    // Reset mutable range for outer inference
                    for operand in &mut v.lowered_func.func.context {
                        operand.identifier.mutable_range =
                            MutableRange { start: InstructionId(0), end: InstructionId(0) };
                        operand.identifier.scope = None;
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}

fn lower_with_mutation_aliasing(func: &mut HIRFunction) -> Result<(), CompilerError> {
    // Phase 1: similar to lower(), but using the new mutation/aliasing inference
    analyse_functions(func)?;
    infer_mutation_aliasing_effects(func, &InferOptions { is_function_expression: true });
    dead_code_elimination(func);
    let function_effects =
        infer_mutation_aliasing_ranges(func, InferRangesOptions { is_function_expression: true });
    rewrite_instruction_kinds_based_on_reassignment(func)?;
    infer_reactive_scope_variables(func);
    func.aliasing_effects = Some(function_effects.clone());

    // Phase 2: populate the Effect of each context variable to use in inferring
    // the outer function. For example, InferMutationAliasingEffects uses context variable
    // effects to decide if the function may be mutable or not.
    let mut captured_or_mutated: FxHashSet<IdentifierId> = FxHashSet::default();
    for effect in &function_effects {
        match effect {
            AliasingEffect::Assign { from, .. }
            | AliasingEffect::Alias { from, .. }
            | AliasingEffect::Capture { from, .. }
            | AliasingEffect::CreateFrom { from, .. }
            | AliasingEffect::MaybeAlias { from, .. } => {
                captured_or_mutated.insert(from.identifier.id);
            }
            AliasingEffect::Mutate { value, .. }
            | AliasingEffect::MutateConditionally { value, .. }
            | AliasingEffect::MutateTransitive { value, .. }
            | AliasingEffect::MutateTransitiveConditionally { value, .. } => {
                captured_or_mutated.insert(value.identifier.id);
            }
            AliasingEffect::Impure { .. }
            | AliasingEffect::Render { .. }
            | AliasingEffect::MutateFrozen { .. }
            | AliasingEffect::MutateGlobal { .. }
            | AliasingEffect::CreateFunction { .. }
            | AliasingEffect::Create { .. }
            | AliasingEffect::Freeze { .. }
            | AliasingEffect::ImmutableCapture { .. }
            | AliasingEffect::Apply { .. } => {
                // no-op
            }
        }
    }

    for operand in &mut func.context {
        if captured_or_mutated.contains(&operand.identifier.id) || operand.effect == Effect::Capture
        {
            operand.effect = Effect::Capture;
        } else {
            operand.effect = Effect::Read;
        }
    }

    Ok(())
}
