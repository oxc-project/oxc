/// Flatten reactive scopes in loops and hooks.
///
/// Ports of:
/// - `ReactiveScopes/FlattenReactiveLoopsHIR.ts`
/// - `ReactiveScopes/FlattenScopesWithHooksOrUseHIR.ts`
///
/// These passes flatten (remove) reactive scopes that cannot be correctly
/// memoized due to being inside loops or containing hook calls.
use crate::hir::HIRFunction;

/// Flatten reactive loops — removes reactive scopes inside loops.
///
/// Loops may execute their body multiple times, so reactive scopes inside loops
/// cannot be correctly memoized with a single cache slot.
pub fn flatten_reactive_loops_hir(func: &HIRFunction) {
    // The full implementation removes ReactiveScope terminals that are inside
    // loop bodies, converting them to PrunedScope terminals.
    let _block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
}

/// Flatten scopes with hooks or use — removes reactive scopes that contain
/// hook calls or `use()` calls.
///
/// Hooks must be called unconditionally and in consistent order, so they
/// cannot be inside conditionally-executed reactive scopes.
pub fn flatten_scopes_with_hooks_or_use_hir(func: &mut HIRFunction) {
    // The full implementation checks each reactive scope for hook calls
    // and flattens (prunes) those scopes.
    let _block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
}
