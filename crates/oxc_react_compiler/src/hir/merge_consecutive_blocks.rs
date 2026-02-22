/// Merge consecutive blocks in the HIR.
///
/// Port of `HIR/MergeConsecutiveBlocks.ts` from the React Compiler.
///
/// Merges sequences of blocks that will always execute consecutively —
/// ie where the predecessor always transfers control to the successor
/// (ie ends in a goto) and where the predecessor is the only predecessor
/// for that successor (ie, there is no other way to reach the successor).
///
/// Note that this pass leaves value/loop blocks alone because they cannot
/// be merged without breaking the structure of the high-level terminals
/// that reference them.
use rustc_hash::{FxHashMap, FxHashSet};

use crate::compiler_error::GENERATED_SOURCE;

use super::hir_builder::mark_predecessors;
use super::hir_types::{
    BlockId, BlockKind, Effect, HIRFunction, Instruction, InstructionValue, LoadLocal, Place,
    Terminal,
};
use super::visitors::terminal_fallthrough;

/// Merge consecutive blocks in the function's HIR.
///
/// # Panics
/// Panics if predecessor data is inconsistent.
pub fn merge_consecutive_blocks(func: &mut HIRFunction) {
    let mut merged = MergedBlocks::new();
    let mut fallthrough_blocks = FxHashSet::default();

    // Collect fallthrough block IDs and recursively merge nested functions
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for &block_id in &block_ids {
        let Some(block) = func.body.blocks.get(&block_id) else { continue };
        if let Some(ft) = terminal_fallthrough(&block.terminal) {
            fallthrough_blocks.insert(ft);
        }

        // Collect indices of instructions that have nested functions
        let nested_indices: Vec<usize> = block
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, instr)| {
                matches!(
                    &instr.value,
                    InstructionValue::FunctionExpression(_) | InstructionValue::ObjectMethod(_)
                )
            })
            .map(|(i, _)| i)
            .collect();

        if nested_indices.is_empty() {
            continue;
        }

        // Extract nested functions, merge them recursively, then put back
        let block = func.body.blocks.get_mut(&block_id);
        if let Some(block) = block {
            for idx in nested_indices {
                match &mut block.instructions[idx].value {
                    InstructionValue::FunctionExpression(v) => {
                        merge_consecutive_blocks(&mut v.lowered_func.func);
                    }
                    InstructionValue::ObjectMethod(v) => {
                        merge_consecutive_blocks(&mut v.lowered_func.func);
                    }
                    _ => {}
                }
            }
        }
    }

    // Process blocks for merging
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for &block_id in &block_ids {
        let (should_merge, predecessor_id) = {
            let Some(block) = func.body.blocks.get(&block_id) else { continue };

            // Can only merge blocks with a single predecessor
            if block.preds.len() != 1 {
                continue;
            }
            // Value blocks cannot merge
            if block.kind != BlockKind::Block {
                continue;
            }
            // Merging across fallthroughs could move the predecessor out of its block scope
            if fallthrough_blocks.contains(&block.id) {
                continue;
            }

            let original_pred_id = *block.preds.iter().next().expect("preds is non-empty");
            let pred_id = merged.get(original_pred_id);

            let Some(pred) = func.body.blocks.get(&pred_id) else { continue };

            // The predecessor must be a goto to this block and be a 'block' kind
            if !matches!(pred.terminal, Terminal::Goto(_)) || pred.kind != BlockKind::Block {
                continue;
            }

            (true, pred_id)
        };

        if !should_merge {
            continue;
        }

        // Remove the block and merge its instructions into the predecessor
        let block = func.body.blocks.remove(&block_id);
        if let Some(block) = block {
            if let Some(predecessor) = func.body.blocks.get_mut(&predecessor_id) {
                // Replace phis in the merged block with canonical assignments to the
                // single operand value (since there is only one predecessor)
                let terminal_id = predecessor.terminal.id();
                for phi in &block.phis {
                    if let Some(operand) = phi.operands.values().next() {
                        let lvalue = Place {
                            identifier: phi.place.identifier.clone(),
                            effect: Effect::ConditionallyMutate,
                            reactive: false,
                            loc: GENERATED_SOURCE,
                        };
                        let instr = Instruction {
                            id: terminal_id,
                            lvalue: lvalue.clone(),
                            value: InstructionValue::LoadLocal(LoadLocal {
                                place: operand.clone(),
                                loc: GENERATED_SOURCE,
                            }),
                            effects: None,
                            loc: GENERATED_SOURCE,
                        };
                        predecessor.instructions.push(instr);
                    }
                }

                predecessor.instructions.extend(block.instructions);
                predecessor.terminal = block.terminal;
            }
            merged.merge(block_id, predecessor_id);
        }
    }

    // Update phi operands with merged block IDs
    let all_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in &all_ids {
        let Some(block) = func.body.blocks.get_mut(block_id) else { continue };
        for phi in &mut block.phis {
            let remapped: Vec<(BlockId, BlockId)> = phi
                .operands
                .keys()
                .filter_map(|&predecessor_id| {
                    let mapped = merged.get(predecessor_id);
                    if mapped == predecessor_id { None } else { Some((predecessor_id, mapped)) }
                })
                .collect();
            for (old_id, new_id) in remapped {
                if let Some(operand) = phi.operands.remove(&old_id) {
                    phi.operands.insert(new_id, operand);
                }
            }
        }
    }

    // Re-mark predecessors
    mark_predecessors(&mut func.body);

    // Update fallthrough references with merged block IDs
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        if let Some(block) = func.body.blocks.get_mut(&block_id) {
            update_terminal_fallthrough(&mut block.terminal, &merged);
        }
    }
}

/// Update fallthrough references in a terminal to use merged block IDs.
fn update_terminal_fallthrough(terminal: &mut Terminal, merged: &MergedBlocks) {
    match terminal {
        Terminal::If(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Branch(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Switch(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::For(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::ForOf(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::ForIn(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::DoWhile(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::While(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Logical(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Ternary(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Optional(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Label(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Sequence(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Try(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Scope(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::PrunedScope(t) => t.fallthrough = merged.get(t.fallthrough),
        Terminal::Unsupported(_)
        | Terminal::Unreachable(_)
        | Terminal::Throw(_)
        | Terminal::Return(_)
        | Terminal::Goto(_)
        | Terminal::MaybeThrow(_) => {}
    }
}

/// Tracks which blocks have been merged into other blocks.
struct MergedBlocks {
    map: FxHashMap<BlockId, BlockId>,
}

impl MergedBlocks {
    fn new() -> Self {
        Self { map: FxHashMap::default() }
    }

    /// Record that `block` was merged into `into`.
    fn merge(&mut self, block: BlockId, into: BlockId) {
        let target = self.get(into);
        self.map.insert(block, target);
    }

    /// Get the id of the block that `block` has been merged into (transitively).
    fn get(&self, block: BlockId) -> BlockId {
        let mut current = block;
        while let Some(&mapped) = self.map.get(&current) {
            current = mapped;
        }
        current
    }
}
