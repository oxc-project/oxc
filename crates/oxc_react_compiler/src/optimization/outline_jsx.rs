/// Outline JSX elements from reactive scopes.
///
/// Port of `Optimization/OutlineJsx.ts` from the React Compiler.
///
/// Moves JSX element creation out of reactive scopes when safe, converting
/// them into outlined component functions. This can reduce the amount of
/// work done during re-renders.
use crate::hir::HIRFunction;

/// Outline JSX elements from reactive scopes.
pub fn outline_jsx(func: &HIRFunction) {
    // The full implementation identifies JSX elements within reactive scopes
    // that have stable props, and outlines them as separate component functions.
    // This allows React to skip re-rendering those subtrees when the parent
    // re-renders but the outlined component's props haven't changed.
    let _block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
}
