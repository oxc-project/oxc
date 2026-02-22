/// Eliminate redundant phi nodes.
///
/// Port of `SSA/EliminateRedundantPhi.ts` from the React Compiler.
///
/// Eliminates redundant phi nodes where:
/// - all operands are the same identifier, ie `x2 = phi(x1, x1, x1)`.
/// - all operands are the same identifier *or* the output of the phi,
///   ie `x2 = phi(x1, x2, x1, x2)`.
///
/// The algorithm is inspired by Braun et al. (2013) but modified to reduce
/// passes over the CFG. We visit blocks in reverse postorder, looping until
/// no new rewrites are found.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::hir::{
    BlockId, HIRFunction, Identifier, IdentifierId, InstructionValue,
    visitors::{map_instruction_lvalues, map_instruction_operands, map_terminal_operands},
};

/// Eliminate redundant phi nodes from the function's HIR.
pub(crate) fn eliminate_redundant_phi(
    func: &mut HIRFunction,
    shared_rewrites: Option<&mut FxHashMap<IdentifierId, Identifier>>,
) {
    let mut owned_rewrites;
    let rewrites: &mut FxHashMap<IdentifierId, Identifier> = if let Some(r) = shared_rewrites {
        r
    } else {
        owned_rewrites = FxHashMap::default();
        &mut owned_rewrites
    };

    let ir = &mut func.body;

    // Track whether the CFG has a back-edge (loop)
    let mut has_back_edge = false;
    let mut visited: FxHashSet<BlockId> = FxHashSet::default();

    let mut size = rewrites.len();
    loop {
        let block_ids: Vec<BlockId> = ir.blocks.keys().copied().collect();
        for block_id in &block_ids {
            let Some(block) = ir.blocks.get_mut(block_id) else { continue };

            // On the first iteration, check for back-edges
            if !has_back_edge {
                for pred_id in &block.preds {
                    if !visited.contains(pred_id) {
                        has_back_edge = true;
                    }
                }
            }
            visited.insert(*block_id);

            // Find redundant phis: drain all phis, process them, put back non-redundant ones
            let phis = std::mem::take(&mut block.phis);
            let mut kept_phis = Vec::with_capacity(phis.len());
            'phis: for mut phi in phis {
                // Rewrite phi operands using current rewrites
                for operand in phi.operands.values_mut() {
                    rewrite_place_id(operand, rewrites);
                }

                // Find if the phi can be eliminated: look for a single unique
                // non-self-referential operand
                let phi_output_id = phi.place.identifier.id;
                let mut same: Option<&Identifier> = None;
                for operand in phi.operands.values() {
                    let op_id = operand.identifier.id;
                    if op_id == phi_output_id {
                        // Operand is the phi itself, skip
                        continue;
                    }
                    if let Some(s) = same {
                        if op_id == s.id {
                            // Same as the previous non-phi operand, skip
                            continue;
                        }
                        // Multiple distinct non-self operands: phi is NOT redundant
                        kept_phis.push(phi);
                        continue 'phis;
                    }
                    same = Some(&operand.identifier);
                }

                // If `same` is Some, the phi is redundant
                if let Some(same_ident) = same {
                    let same_cloned = same_ident.clone();
                    rewrites.insert(phi_output_id, same_cloned);
                    // Do NOT push phi back: it's been eliminated
                } else {
                    // All operands were self-referential or phi was empty; keep it
                    kept_phis.push(phi);
                }
            }
            block.phis = kept_phis;
        }

        // Rewrite instruction lvalues and operands
        for block_id in &block_ids {
            let Some(block) = ir.blocks.get_mut(block_id) else { continue };

            for instr in &mut block.instructions {
                // Rewrite lvalues
                map_instruction_lvalues(instr, &mut |mut place| {
                    rewrite_place_id(&mut place, rewrites);
                    place
                });
                // Rewrite operands
                map_instruction_operands(instr, &mut |mut place| {
                    rewrite_place_id(&mut place, rewrites);
                    place
                });

                // Recursively handle nested functions
                match &mut instr.value {
                    InstructionValue::FunctionExpression(v) => {
                        for ctx_place in &mut v.lowered_func.func.context {
                            rewrite_place_id(ctx_place, rewrites);
                        }
                        eliminate_redundant_phi(&mut v.lowered_func.func, Some(rewrites));
                    }
                    InstructionValue::ObjectMethod(v) => {
                        for ctx_place in &mut v.lowered_func.func.context {
                            rewrite_place_id(ctx_place, rewrites);
                        }
                        eliminate_redundant_phi(&mut v.lowered_func.func, Some(rewrites));
                    }
                    _ => {}
                }
            }

            // Rewrite terminal operands
            map_terminal_operands(&mut block.terminal, &mut |mut place| {
                rewrite_place_id(&mut place, rewrites);
                place
            });
        }

        // Only loop if there were new rewrites and the CFG has loops
        if rewrites.len() <= size || !has_back_edge {
            break;
        }
        size = rewrites.len();
    }
}

/// Rewrite a place's identifier if it has a mapping in the rewrites table.
fn rewrite_place_id(place: &mut crate::hir::Place, rewrites: &FxHashMap<IdentifierId, Identifier>) {
    if let Some(rewrite) = rewrites.get(&place.identifier.id) {
        place.identifier = rewrite.clone();
    }
}
