/// Collect hoistable property loads.
///
/// Port of `HIR/CollectHoistablePropertyLoads.ts` from the React Compiler.
///
/// Uses control flow graph analysis to determine which identifiers can be
/// assumed to be non-null objects on a per-block basis. This enables hoisting
/// property loads out of conditionals when it's safe to do so.
///
/// Example:
/// ```js
/// if (...) {
///   read(x.a); // x is known non-null after this point
///   return;
/// }
/// read(y.b); // y is known non-null in this block
/// ```
use rustc_hash::{FxHashMap, FxHashSet};

use super::hir_types::{
    BlockId, HIRFunction, IdentifierId, InstructionValue,
};

/// Result of hoistable property load analysis.
pub struct HoistablePropertyLoads {
    /// Map from block ID to the set of identifiers known to be non-null at that block.
    pub non_null_by_block: FxHashMap<BlockId, FxHashSet<IdentifierId>>,
}

/// Collect hoistable property loads for the given function.
pub fn collect_hoistable_property_loads(func: &HIRFunction) -> HoistablePropertyLoads {
    let mut non_null_by_block: FxHashMap<BlockId, FxHashSet<IdentifierId>> = FxHashMap::default();

    // Phase 1: Collect property loads per block
    // When a property is loaded from an object, that object must be non-null
    for (&block_id, block) in &func.body.blocks {
        let mut non_null = FxHashSet::default();
        for instr in &block.instructions {
            match &instr.value {
                InstructionValue::PropertyLoad(v) => {
                    non_null.insert(v.object.identifier.id);
                }
                InstructionValue::PropertyStore(v) => {
                    non_null.insert(v.object.identifier.id);
                }
                InstructionValue::ComputedLoad(v) => {
                    non_null.insert(v.object.identifier.id);
                }
                InstructionValue::ComputedStore(v) => {
                    non_null.insert(v.object.identifier.id);
                }
                InstructionValue::MethodCall(v) => {
                    non_null.insert(v.receiver.identifier.id);
                }
                _ => {}
            }
        }
        non_null_by_block.insert(block_id, non_null);
    }

    // Phase 2: Forward propagation — intersect non-null sets along control flow
    // A block's non-null set includes identifiers that are non-null in ALL predecessors
    let mut changed = true;
    while changed {
        changed = false;
        let block_ids: Vec<BlockId> = func.body.blocks.keys().copied().collect();
        for &block_id in &block_ids {
            let Some(block) = func.body.blocks.get(&block_id) else { continue };

            if block.preds.is_empty() {
                continue;
            }

            // Intersect non-null sets from all predecessors
            let mut intersection: Option<FxHashSet<IdentifierId>> = None;
            for &pred_id in &block.preds {
                if let Some(pred_non_null) = non_null_by_block.get(&pred_id) {
                    intersection = Some(match intersection {
                        None => pred_non_null.clone(),
                        Some(existing) => existing
                            .intersection(pred_non_null)
                            .copied()
                            .collect(),
                    });
                }
            }

            if let Some(pred_intersection) = intersection {
                let current = non_null_by_block.entry(block_id).or_default();
                for id in &pred_intersection {
                    if current.insert(*id) {
                        changed = true;
                    }
                }
            }
        }
    }

    HoistablePropertyLoads { non_null_by_block }
}
