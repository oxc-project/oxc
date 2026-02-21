/// Prune unused labels in the HIR.
///
/// Port of `HIR/PruneUnusedLabelsHIR.ts` from the React Compiler.
///
/// Removes label terminals where the labeled block simply jumps to the
/// fallthrough, merging the label, block, and fallthrough into one block.
use rustc_hash::FxHashMap;

use super::hir_types::{BlockId, BlockKind, GotoVariant, HIRFunction, Terminal};

/// Prune unused labels from the function's HIR.
pub fn prune_unused_labels_hir(func: &mut HIRFunction) {
    // Phase 1: Find label terminals that can be merged
    let mut merged: Vec<(BlockId, BlockId, BlockId)> = Vec::new(); // (label, next, fallthrough)

    for (&block_id, block) in &func.body.blocks {
        if let Terminal::Label(label_term) = &block.terminal {
            let next_id = label_term.block;
            let fallthrough_id = label_term.fallthrough;

            let next = func.body.blocks.get(&next_id);
            let fallthrough = func.body.blocks.get(&fallthrough_id);

            if let (Some(next), Some(fallthrough)) = (next, fallthrough) {
                // Check if the next block is a simple goto to the fallthrough
                if let Terminal::Goto(goto) = &next.terminal {
                    if goto.variant == GotoVariant::Break && goto.block == fallthrough_id
                        && next.kind == BlockKind::Block && fallthrough.kind == BlockKind::Block
                    {
                        merged.push((block_id, next_id, fallthrough_id));
                    }
                }
            }
        }
    }

    // Phase 2: Perform the merges
    let mut rewrites: FxHashMap<BlockId, BlockId> = FxHashMap::default();
    for (original_label_id, next_id, fallthrough_id) in &merged {
        let label_id = rewrites.get(original_label_id).copied().unwrap_or(*original_label_id);

        // Get the instructions and terminal from next and fallthrough blocks
        let next_instrs = func
            .body
            .blocks
            .get(next_id)
            .map(|b| b.instructions.clone())
            .unwrap_or_default();
        let fallthrough_data = func.body.blocks.get(fallthrough_id).map(|b| {
            (b.instructions.clone(), b.terminal.clone())
        });

        if let Some((ft_instrs, ft_terminal)) = fallthrough_data {
            if let Some(label) = func.body.blocks.get_mut(&label_id) {
                label.instructions.extend(next_instrs);
                label.instructions.extend(ft_instrs);
                label.terminal = ft_terminal;
            }

            func.body.blocks.remove(next_id);
            func.body.blocks.remove(fallthrough_id);
            rewrites.insert(*fallthrough_id, label_id);
        }
    }

    // Phase 3: Update predecessor references
    let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        if let Some(block) = func.body.blocks.get_mut(&block_id) {
            let preds_to_rewrite: Vec<(BlockId, BlockId)> = block
                .preds
                .iter()
                .filter_map(|pred| rewrites.get(pred).map(|&new| (*pred, new)))
                .collect();
            for (old, new) in preds_to_rewrite {
                block.preds.remove(&old);
                block.preds.insert(new);
            }
        }
    }
}
