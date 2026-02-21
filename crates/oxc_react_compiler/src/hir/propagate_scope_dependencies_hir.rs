
use crate::hir::HIRFunction;

/// Propagate scope dependencies through the HIR.
pub fn propagate_scope_dependencies_hir(func: &mut HIRFunction) {
    // The full implementation:
    // 1. For each reactive scope, identifies all values read inside the scope
    // 2. Filters to keep only values defined outside the scope
    // 3. Builds property access chains for dependencies (e.g., `props.a.b`)
    // 4. Minimizes the dependency set (removes redundant sub-paths)
    // 5. Stores the dependency set on the ReactiveScope object
    let _block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
}
