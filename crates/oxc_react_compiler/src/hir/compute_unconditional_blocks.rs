/// Compute the set of unconditionally executed blocks.
///
/// Port of `HIR/ComputeUnconditionalBlocks.ts` from the React Compiler.
///
/// Uses the post-dominator tree to find blocks that are always reachable
/// from the entry block (the set of blocks that unconditionally execute).
use rustc_hash::FxHashSet;

use super::{
    hir_types::{BlockId, HIRFunction},
    dominator::compute_post_dominator_tree,
};

/// Compute the set of blocks that unconditionally execute from the entry block.
pub fn compute_unconditional_blocks(func: &HIRFunction) -> FxHashSet<BlockId> {
    let mut unconditional_blocks = FxHashSet::default();
    let dominators = compute_post_dominator_tree(func, false);
    let exit = dominators.exit();

    let mut current: Option<BlockId> = Some(func.body.entry);
    while let Some(block_id) = current {
        if block_id == exit {
            break;
        }
        if !unconditional_blocks.insert(block_id) {
            // Non-terminating loop — should not happen with a valid post-dominator tree
            break;
        }
        current = dominators.get(block_id);
    }

    unconditional_blocks
}
