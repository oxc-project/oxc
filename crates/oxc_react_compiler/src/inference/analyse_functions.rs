/// Analyse nested function expressions for mutation and aliasing effects.
///
/// Port of `Inference/AnalyseFunctions.ts` from the React Compiler.
///
/// For each nested function expression or object method, this pass:
/// 1. Recursively analyses inner functions
/// 2. Infers mutation/aliasing effects
/// 3. Runs DCE and reactive scope inference
/// 4. Populates context variable effects for the outer function
use crate::hir::{
    Effect, HIRFunction, IdentifierId, InstructionId, InstructionValue, MutableRange,
};
use rustc_hash::FxHashSet;

/// Analyse all nested function expressions in the given function.
pub fn analyse_functions(func: &mut HIRFunction) {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let block = match func.body.blocks.get_mut(&block_id) {
            Some(b) => b,
            None => continue,
        };

        for instr in &mut block.instructions {
            match &mut instr.value {
                InstructionValue::FunctionExpression(v) => {
                    lower_with_mutation_aliasing(&mut v.lowered_func.func);

                    // Reset mutable range for outer inference
                    for operand in &mut v.lowered_func.func.context {
                        operand.identifier.mutable_range = MutableRange {
                            start: InstructionId(0),
                            end: InstructionId(0),
                        };
                        operand.identifier.scope = None;
                    }
                }
                InstructionValue::ObjectMethod(v) => {
                    lower_with_mutation_aliasing(&mut v.lowered_func.func);

                    // Reset mutable range for outer inference
                    for operand in &mut v.lowered_func.func.context {
                        operand.identifier.mutable_range = MutableRange {
                            start: InstructionId(0),
                            end: InstructionId(0),
                        };
                        operand.identifier.scope = None;
                    }
                }
                _ => {}
            }
        }
    }
}

fn lower_with_mutation_aliasing(func: &mut HIRFunction) {
    // Phase 1: Recursive analysis
    analyse_functions(func);

    // The full implementation would call:
    // - inferMutationAliasingEffects
    // - deadCodeElimination
    // - inferMutationAliasingRanges
    // - rewriteInstructionKindsBasedOnReassignment
    // - inferReactiveScopeVariables

    // Phase 2: Populate context variable effects
    // For now, set all context variables to Read effect
    // (the full implementation would analyze the aliasing effects)
    let captured_or_mutated: FxHashSet<IdentifierId> = FxHashSet::default();

    for operand in &mut func.context {
        if captured_or_mutated.contains(&operand.identifier.id)
            || operand.effect == Effect::Capture
        {
            operand.effect = Effect::Capture;
        } else {
            operand.effect = Effect::Read;
        }
    }
}
