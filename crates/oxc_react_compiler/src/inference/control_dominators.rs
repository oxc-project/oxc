/// Control dominators — identify blocks controlled by specific values.
///
/// Port of `Inference/ControlDominators.ts` from the React Compiler.
///
/// Returns a function that lazily determines whether particular blocks are
/// "controlled" by values of interest (e.g., reactive values that determine
/// whether code is conditionally executed).
use rustc_hash::FxHashSet;

use crate::hir::{
    BlockId, HIRFunction, Place, Terminal,
    dominator::{PostDominator, compute_post_dominator_tree},
};

/// Check whether a block is controlled by a value of interest.
///
/// Uses post-dominator frontier analysis to find the earliest branch points
/// from which execution may or may not reach a given block.
pub fn create_control_dominators<'a>(
    func: &'a HIRFunction,
    is_control_variable: &'a dyn Fn(&Place) -> bool,
) -> impl Fn(BlockId) -> bool + 'a {
    let post_dominators = compute_post_dominator_tree(func, false);

    move |block_id: BlockId| -> bool {
        let control_blocks = post_dominator_frontier(func, &post_dominators, block_id);
        for &control_block_id in &control_blocks {
            let control_block = match func.body.blocks.get(&control_block_id) {
                Some(b) => b,
                None => continue,
            };
            match &control_block.terminal {
                Terminal::If(t) => {
                    if is_control_variable(&t.test) {
                        return true;
                    }
                }
                Terminal::Branch(t) => {
                    if is_control_variable(&t.test) {
                        return true;
                    }
                }
                Terminal::Switch(t) => {
                    if is_control_variable(&t.test) {
                        return true;
                    }
                    for case in &t.cases {
                        if let Some(test) = &case.test
                            && is_control_variable(test) {
                                return true;
                            }
                    }
                }
                _ => {}
            }
        }
        false
    }
}

/// Compute the post-dominator frontier of a block.
///
/// These are the earliest blocks from which execution branches such that
/// it may or may not reach the target block.
fn post_dominator_frontier(
    func: &HIRFunction,
    post_dominators: &PostDominator,
    target_id: BlockId,
) -> FxHashSet<BlockId> {
    let target_post_dominators = post_dominators_of(func, post_dominators, target_id);
    let mut visited = FxHashSet::default();
    let mut frontier = FxHashSet::default();

    let all_blocks: Vec<BlockId> = target_post_dominators
        .iter()
        .copied()
        .chain(std::iter::once(target_id))
        .collect();

    for block_id in all_blocks {
        if !visited.insert(block_id) {
            continue;
        }
        let block = match func.body.blocks.get(&block_id) {
            Some(b) => b,
            None => continue,
        };
        for &pred in &block.preds {
            if !target_post_dominators.contains(&pred) {
                frontier.insert(pred);
            }
        }
    }

    frontier
}

/// Compute all blocks that post-dominate the target block.
fn post_dominators_of(
    func: &HIRFunction,
    post_dominators: &PostDominator,
    target_id: BlockId,
) -> FxHashSet<BlockId> {
    let mut result = FxHashSet::default();
    let mut visited = FxHashSet::default();
    let mut queue = vec![target_id];

    while let Some(current_id) = queue.pop() {
        if !visited.insert(current_id) {
            continue;
        }
        let current = match func.body.blocks.get(&current_id) {
            Some(b) => b,
            None => continue,
        };
        for &pred in &current.preds {
            let pred_post_dom = post_dominators.get(pred).unwrap_or(pred);
            if pred_post_dom == target_id || result.contains(&pred_post_dom) {
                result.insert(pred);
            }
            queue.push(pred);
        }
    }

    result
}
