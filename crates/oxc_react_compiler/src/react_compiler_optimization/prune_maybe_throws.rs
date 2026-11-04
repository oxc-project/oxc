// Copyright (c) Meta Platforms, Inc. and affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

//! Prunes `MaybeThrow` terminals for blocks that can provably never throw.
//!
//! Currently conservative: only removes handlers when every instruction is known not to throw.
//! Even a variable reference could throw due to TDZ.
//!
//! Analogous to TS `Optimization/PruneMaybeThrows.ts`.

use oxc_index::IndexVec;
use rustc_hash::FxHashSet;

use oxc_diagnostics::OxcDiagnostic;

use crate::diagnostics;
use crate::react_compiler_hir::environment::Environment;
use crate::react_compiler_hir::{
    ArrayElement, BlockId, HirFunction, InstructionValue, ObjectPropertyKey,
    ObjectPropertyOrSpread, Terminal, assert_consistent_identifiers,
    assert_terminal_successors_exist,
};
use crate::react_compiler_lowering::{
    get_reverse_postordered_blocks, mark_instruction_ids, mark_predecessors,
    remove_dead_do_while_statements, remove_unnecessary_try_catch, remove_unreachable_for_updates,
};

use crate::react_compiler_optimization::merge_consecutive_blocks::merge_consecutive_blocks;
use crate::react_compiler_ssa::eliminate_redundant_phi;

/// Prune `MaybeThrow` terminals for blocks that cannot throw, then clean up the CFG.
pub fn prune_maybe_throws<'a>(
    func: &mut HirFunction<'a>,
    env: &mut Environment<'a>,
) -> Result<(), OxcDiagnostic> {
    let terminal_mapping = prune_maybe_throws_impl(func);
    if let Some(terminal_mapping) = terminal_mapping {
        // If terminals have changed then blocks may have become newly unreachable.
        // Re-run minification of the graph (incl reordering instruction ids).
        func.body.blocks = get_reverse_postordered_blocks(&func.body, env.allocator);
        remove_unreachable_for_updates(&mut func.body);
        remove_dead_do_while_statements(&mut func.body);
        remove_unnecessary_try_catch(&mut func.body);

        // Pruning a handler makes the catch block, and every block only it reaches,
        // unreachable. Divergence from TS, which keeps references to those blocks:
        // - Recompute predecessors, so the join after the `try` no longer counts
        //   the removed blocks and can be merged into its only real predecessor.
        //   A stale count leaves it unmerged as the target of a `break` that no
        //   longer has a matching `try` fallthrough, and the code after the `try`
        //   is silently dropped from the output.
        // - Prune their phi operands and eliminate the phis this makes redundant,
        //   as constant propagation does. Otherwise the join is left with a single
        //   predecessor but a multi-operand phi, which MergeConsecutiveBlocks rejects.
        mark_predecessors(&mut func.body);
        if prune_unreachable_phi_operands(func) {
            eliminate_redundant_phi(func, env);
        }

        mark_instruction_ids(&mut func.body, &mut func.instructions);
        merge_consecutive_blocks(func, &mut env.functions, env.allocator)?;

        // Rewrite phi operands to reference the updated predecessor blocks
        for block in func.body.blocks.values_mut() {
            let preds = &block.preds;
            let mut phi_updates: Vec<(usize, Vec<(BlockId, BlockId)>)> = Vec::new();

            for (phi_idx, phi) in block.phis.iter().enumerate() {
                let mut updates = Vec::new();
                for (predecessor, _) in &phi.operands {
                    if !preds.contains(predecessor) {
                        let mapped_terminal =
                            terminal_mapping.get(*predecessor).copied().flatten().ok_or_else(
                                || {
                                    diagnostics::missing_phi_predecessor_mapping(
                                        predecessor.index(),
                                        block.id.index(),
                                    )
                                },
                            )?;
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

        assert_consistent_identifiers(func, &env.identifiers)?;
        assert_terminal_successors_exist(func)?;
    }
    Ok(())
}

/// Remove phi operands whose predecessor block is no longer in the CFG.
/// Returns whether any operand was removed.
fn prune_unreachable_phi_operands(func: &mut HirFunction) -> bool {
    let blocks = &mut func.body.blocks;
    let live_blocks: FxHashSet<BlockId> = blocks.keys().copied().collect();
    let mut pruned = false;
    for block in blocks.values_mut() {
        for phi in &mut block.phis {
            let len = phi.operands.len();
            phi.operands.retain(|pred, _operand| live_blocks.contains(pred));
            pruned |= phi.operands.len() != len;
        }
    }
    pruned
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
            .any(|instr_id| value_may_throw_when_pruning(&instructions[instr_id.index()].value));

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

/// Returns whether a `MaybeThrow` edge must be retained during CFG pruning.
///
/// Local stores cannot throw by themselves, but value blocks end with a store.
/// Removing that final handler edge can reorder the catch block ahead of the
/// successful fallthrough, causing reactive scopes to span both paths.
fn value_may_throw_when_pruning(value: &InstructionValue) -> bool {
    matches!(value, InstructionValue::StoreLocal { .. } | InstructionValue::StoreContext { .. })
        || value_may_throw(value)
}

/// Returns whether evaluating an instruction value can throw.
///
/// Operand evaluation is represented by separate HIR instructions, so plain array/object
/// construction is safe once its operands exist. Spreads and computed object keys remain part of
/// the construction operation and can invoke user code.
pub(crate) fn value_may_throw(value: &InstructionValue) -> bool {
    match value {
        InstructionValue::DeclareLocal { .. }
        | InstructionValue::DeclareContext { .. }
        | InstructionValue::Primitive { .. }
        | InstructionValue::JSXText { .. }
        | InstructionValue::TypeCastExpression { .. }
        | InstructionValue::ObjectMethod { .. }
        | InstructionValue::FunctionExpression { .. }
        | InstructionValue::RegExpLiteral { .. }
        | InstructionValue::MetaProperty { .. }
        | InstructionValue::Debugger { .. }
        | InstructionValue::StartMemoize { .. }
        | InstructionValue::FinishMemoize { .. } => false,
        InstructionValue::StoreLocal { lvalue, .. }
        | InstructionValue::StoreContext { lvalue, .. } => {
            lvalue.kind == crate::react_compiler_hir::InstructionKind::Reassign
        }
        InstructionValue::ArrayExpression { elements, .. } => {
            elements.iter().any(|element| matches!(element, ArrayElement::Spread(_)))
        }
        InstructionValue::ObjectExpression { properties, .. } => {
            properties.iter().any(|property| match property {
                ObjectPropertyOrSpread::Property(property) => {
                    matches!(property.key, ObjectPropertyKey::Computed { .. })
                }
                ObjectPropertyOrSpread::Spread(_) => true,
            })
        }
        _ => true,
    }
}
