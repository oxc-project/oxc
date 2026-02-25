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
    BlockId, HIRFunction, IdentifierId, InstructionValue, ManualMemoDependencyRoot,
    ReactFunctionType, ReactiveParam,
};

/// Result of hoistable property load analysis.
pub struct HoistablePropertyLoads {
    /// Map from block ID to the set of identifiers known to be non-null at that block.
    pub non_null_by_block: FxHashMap<BlockId, FxHashSet<IdentifierId>>,
}

/// Collect hoistable property loads for the given function.
pub fn collect_hoistable_property_loads(func: &HIRFunction) -> HoistablePropertyLoads {
    let mut non_null_by_block: FxHashMap<BlockId, FxHashSet<IdentifierId>> = FxHashMap::default();

    // Collect known-immutable (always non-null) identifiers.
    //
    // Port of the TS `knownImmutableIdentifiers` / `knownNonNullIdentifiers`:
    // For Component and Hook functions, params are always non-null objects (e.g. `props`).
    // Adding them here means they appear in every block's non-null set, allowing
    // property paths like `props.value` to be tracked as dependencies without truncation.
    let mut known_non_null: FxHashSet<IdentifierId> = FxHashSet::default();
    if std::env::var("DEBUG_HOISTABLE").is_ok() {
        eprintln!("[HOISTABLE] fn_type={:?} params_len={}", func.fn_type, func.params.len());
    }
    if matches!(func.fn_type, ReactFunctionType::Component | ReactFunctionType::Hook) {
        for param in &func.params {
            match param {
                ReactiveParam::Place(p) => {
                    if std::env::var("DEBUG_HOISTABLE").is_ok() {
                        eprintln!(
                            "[HOISTABLE] adding param id={:?} name={:?} as known_non_null",
                            p.identifier.id, p.identifier.name
                        );
                    }
                    known_non_null.insert(p.identifier.id);
                }
                ReactiveParam::Spread(s) => {
                    known_non_null.insert(s.place.identifier.id);
                }
            }
        }
    }

    // Phase 1: Collect property loads per block.
    // When a property is loaded from an object, that object must be non-null.
    // Also collect from StartMemoize deps (for preserveExistingMemoizationGuarantees):
    // any non-optional path prefix in a StartMemoize dep implies the root is non-null.
    for (&block_id, block) in &func.body.blocks {
        // Start each block with the known-immutable non-null identifiers
        let mut non_null = known_non_null.clone();

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
                InstructionValue::StartMemoize(v) => {
                    // Port of the TS `enablePreserveExistingMemoizationGuarantees` branch:
                    // For each dep in StartMemoize.deps, add the root identifier as non-null
                    // for each non-optional path prefix. This allows property paths like
                    // `props.value` specified in a useMemo deps array to propagate correctly.
                    if let Some(deps) = &v.deps {
                        if func.env.config.enable_preserve_existing_memoization_guarantees {
                            for dep in deps {
                                if let ManualMemoDependencyRoot::NamedLocal { value, .. } =
                                    &dep.root
                                {
                                    // Add the root identifier for each non-optional path prefix.
                                    // e.g. for `props.value`, add `props` (prefix of length 0).
                                    for path_entry in &dep.path {
                                        if path_entry.optional {
                                            break;
                                        }
                                        // The root identifier (props) is added as non-null
                                        // for this block since it must be non-null to access
                                        // any of its properties.
                                        non_null.insert(value.identifier.id);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        non_null_by_block.insert(block_id, non_null);
    }

    // Phase 2: Forward propagation — intersect non-null sets along control flow.
    // A block's non-null set includes identifiers that are non-null in ALL predecessors.
    // Note: known_non_null identifiers are already in every block, so they stay non-null
    // throughout propagation.
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
                        Some(existing) => existing.intersection(pred_non_null).copied().collect(),
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
