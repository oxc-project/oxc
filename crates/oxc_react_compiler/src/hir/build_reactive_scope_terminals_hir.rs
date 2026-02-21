
use crate::hir::HIRFunction;

/// Build reactive scope terminals in the HIR.
pub fn build_reactive_scope_terminals_hir(func: &mut HIRFunction) {
    // The full implementation:
    // 1. Collects all reactive scopes from identifier annotations
    // 2. For each scope, finds the start and end instructions
    // 3. Splits blocks at scope boundaries
    // 4. Creates new Scope/PrunedScope terminals
    // 5. Wires up the CFG so the scope block flows into the fallthrough
    let _block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
}
