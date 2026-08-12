// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Prunes `MaybeThrow` terminals for blocks that can provably never throw.
//!
//! Currently very conservative: only affects blocks with primitives or
//! array/object literals. Even a variable reference could throw due to TDZ.
//!
//! Analogous to TS `Optimization/PruneMaybeThrows.ts`.

use oxc_allocator::Allocator;
use oxc_index::{IndexSlice, IndexVec};

use oxc_diagnostics::OxcDiagnostic;

use crate::diagnostics::ErrorCategory;
use crate::react_compiler_hir::{
    BlockId, FunctionId, HirFunction, Instruction, InstructionValue, Terminal,
};
use crate::react_compiler_lowering::{
    get_reverse_postordered_blocks, mark_instruction_ids, remove_dead_do_while_statements,
    remove_unnecessary_try_catch, remove_unreachable_for_updates,
};

use crate::react_compiler_optimization::merge_consecutive_blocks::merge_consecutive_blocks;

/// Prune `MaybeThrow` terminals for blocks that cannot throw, then clean up the CFG.
pub fn prune_maybe_throws<'a>(
    func: &mut HirFunction<'a>,
    functions: &mut IndexSlice<FunctionId, [HirFunction<'a>]>,
    alloc: &'a Allocator,
) -> Result<(), OxcDiagnostic> {
    let terminal_mapping = prune_maybe_throws_impl(func);
    if let Some(terminal_mapping) = terminal_mapping {
        // If terminals have changed then blocks may have become newly unreachable.
        // Re-run minification of the graph (incl reordering instruction ids).
        func.body.blocks = get_reverse_postordered_blocks(&func.body, alloc);
        remove_unreachable_for_updates(&mut func.body);
        remove_dead_do_while_statements(&mut func.body);
        remove_unnecessary_try_catch(&mut func.body);
        mark_instruction_ids(&mut func.body, &mut func.instructions);
        merge_consecutive_blocks(func, functions, alloc);

        // Rewrite phi operands to reference the updated predecessor blocks
        for block in func.body.blocks.values_mut() {
            let preds = &block.preds;
            let mut phi_updates: Vec<(usize, Vec<(BlockId, BlockId)>)> = Vec::new();

            for (phi_idx, phi) in block.phis.iter().enumerate() {
                let mut updates = Vec::new();
                for (predecessor, _) in &phi.operands {
                    if !preds.contains(predecessor) {
                        let mapped_terminal =
                            terminal_mapping.get(*predecessor).copied().flatten().ok_or_else(|| {
                                ErrorCategory::Invariant
                                    .diagnostic("Expected non-existing phi operand's predecessor to have been mapped to a new terminal")
                                    .with_help(format!(
                                        "Could not find mapping for predecessor bb{} in block bb{}",
                                        predecessor.index(), block.id.index(),
                                    ))
                            })?;
                        updates.push((*predecessor, mapped_terminal));
                    }
                }
                if !updates.is_empty() {
                    phi_updates.push((phi_idx, updates));
                }
            }

            for (phi_idx, updates) in phi_updates {
                for (old_pred, new_pred) in updates {
                    let operand = block.phis[phi_idx].operands.shift_remove(&old_pred).unwrap();
                    block.phis[phi_idx].operands.insert(new_pred, operand);
                }
            }
        }
    }
    Ok(())
}

fn prune_maybe_throws_impl(func: &mut HirFunction) -> Option<IndexVec<BlockId, Option<BlockId>>> {
    // Both keys (continuations) and values (source blocks) are ids of blocks present
    // in the function body, so the maximum present id bounds the id space.
    let num_ids = func.body.blocks.keys().map(|id| id.index() + 1).max().unwrap_or(0);
    let mut terminal_mapping: IndexVec<BlockId, Option<BlockId>> =
        IndexVec::from_vec(vec![None; num_ids]);
    let mut mapped_any = false;
    let instructions = &func.instructions;

    for block in func.body.blocks.values_mut() {
        let continuation = match &block.terminal {
            Terminal::MaybeThrow { continuation, .. } => *continuation,
            _ => continue,
        };

        let can_throw = block
            .instructions
            .iter()
            .any(|instr_id| instruction_may_throw(&instructions[instr_id.index()]));

        if !can_throw {
            let source = terminal_mapping[block.id].unwrap_or(block.id);
            terminal_mapping[continuation] = Some(source);
            mapped_any = true;
            // Null out the handler rather than replacing with Goto.
            // Preserving the MaybeThrow makes the continuations clear for
            // BuildReactiveFunction, while nulling out the handler tells us
            // that control cannot flow to the handler.
            if let Terminal::MaybeThrow { handler, .. } = &mut block.terminal {
                *handler = None;
            }
        }
    }

    if mapped_any { Some(terminal_mapping) } else { None }
}

fn instruction_may_throw(instr: &Instruction) -> bool {
    !matches!(
        &instr.value,
        InstructionValue::Primitive { .. }
            | InstructionValue::ArrayExpression { .. }
            | InstructionValue::ObjectExpression { .. }
    )
}
