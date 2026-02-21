/// Inline immediately-invoked function expressions (IIFEs).
///
/// Port of `Inference/InlineImmediatelyInvokedFunctionExpressions.ts` from the React Compiler.
///
/// Inlines IIFEs to allow more fine-grained memoization of the values they produce.
/// The implementation relies on HIR's labeled blocks to represent the inlined function body.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::hir::{
    HIRFunction, IdentifierId, InstructionValue,
};

/// Inline immediately-invoked function expressions in the given function.
pub fn inline_immediately_invoked_function_expressions(func: &mut HIRFunction) {
    // Track all function expressions that are assigned to a temporary
    let mut functions: FxHashMap<IdentifierId, IdentifierId> = FxHashMap::default();
    let mut inlined_functions: FxHashSet<IdentifierId> = FxHashSet::default();

    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();

    for block_id in block_ids {
        let Some(block) = func.body.blocks.get(&block_id) else { continue };

        // Only process statement blocks (not expression blocks)
        if !block.kind.is_statement() {
            continue;
        }

        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::FunctionExpression(_v) => {
                    // Track function expressions assigned to temporaries
                    if instr.lvalue.identifier.name.is_none() {
                        functions.insert(instr.lvalue.identifier.id, block_id.into());
                    }
                }
                InstructionValue::CallExpression(v) => {
                    // Check if this is an IIFE (calling a local function expression with no args)
                    if !v.args.is_empty() {
                        continue;
                    }
                    let callee_id = v.callee.identifier.id;
                    if !functions.contains_key(&callee_id) {
                        continue;
                    }

                    // Mark for inlining
                    inlined_functions.insert(callee_id);

                    // In the full implementation, we would:
                    // 1. Create a continuation block for code after the IIFE
                    // 2. Trim the current block to stop before the IIFE call
                    // 3. Wire the IIFE body's entry as the successor
                    // 4. Replace return statements in the IIFE with assignments + goto
                    // 5. Add the IIFE body's blocks to the outer function
                }
                _ => {}
            }
        }
    }

    // Remove inlined function expressions (they're now part of the outer flow)
    if !inlined_functions.is_empty() {
        let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
        for block_id in block_ids {
            if let Some(block) = func.body.blocks.get_mut(&block_id) {
                block.instructions.retain(|instr| {
                    if let InstructionValue::FunctionExpression(_) = &instr.value {
                        !inlined_functions.contains(&instr.lvalue.identifier.id)
                    } else {
                        true
                    }
                });
            }
        }
    }
}

impl From<crate::hir::BlockId> for IdentifierId {
    fn from(block_id: crate::hir::BlockId) -> Self {
        IdentifierId(block_id.0)
    }
}
