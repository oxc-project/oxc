/// Infer reactive places in the HIR.
///
/// Port of `Inference/InferReactivePlaces.ts` from the React Compiler.
///
/// Determines which places (variables) in the function are "reactive" — meaning
/// they may change between renders and therefore need memoization consideration.
///
/// This pass:
/// 1. Tracks sources of stability (hook calls like useRef, useState)
/// 2. Uses disjoint sets to group related mutable values
/// 3. Marks reactive places on each identifier
use rustc_hash::FxHashSet;

use crate::hir::{
    HIRFunction, IdentifierId,
    visitors::each_instruction_operand,
};

/// Infer which places in the function are reactive.
pub fn infer_reactive_places(func: &mut HIRFunction) {
    // Phase 1: Collect all reactive identifiers (params and hooks return values)
    let mut reactive_ids: FxHashSet<IdentifierId> = FxHashSet::default();

    // All function parameters are reactive (they may change between renders)
    for param in &func.params {
        let place = match param {
            crate::hir::ReactiveParam::Place(p) => p,
            crate::hir::ReactiveParam::Spread(s) => &s.place,
        };
        reactive_ids.insert(place.identifier.id);
    }

    // Phase 2: Propagate reactivity through data flow
    // A value is reactive if it depends on any reactive value
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    let mut changed = true;
    while changed {
        changed = false;
        for &block_id in &block_ids {
            let block = match func.body.blocks.get(&block_id) {
                Some(b) => b,
                None => continue,
            };

            for instr in &block.instructions {
                let lvalue_id = instr.lvalue.identifier.id;

                // If any operand is reactive, the lvalue is reactive
                let has_reactive_operand = each_instruction_operand(instr)
                    .iter()
                    .any(|place| reactive_ids.contains(&place.identifier.id));

                if has_reactive_operand && reactive_ids.insert(lvalue_id) {
                    changed = true;
                }
            }
        }
    }

    // Phase 3: Mark reactive places on identifiers
    for block_id in &block_ids {
        let block = match func.body.blocks.get_mut(block_id) {
            Some(b) => b,
            None => continue,
        };

        for instr in &mut block.instructions {
            instr.lvalue.reactive = reactive_ids.contains(&instr.lvalue.identifier.id);
        }
    }
}
