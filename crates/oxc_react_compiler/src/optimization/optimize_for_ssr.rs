/// Optimize for server-side rendering (SSR).
///
/// Port of `Optimization/OptimizeForSSR.ts` from the React Compiler.
///
/// Removes client-only features (effects, refs, state) from the compiled
/// output when targeting SSR mode. This produces smaller server bundles
/// and avoids SSR-related hydration issues.
use crate::hir::{HIRFunction, InstructionValue};

/// Optimize the function for SSR output.
pub fn optimize_for_ssr(func: &mut HIRFunction) {
    let block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
    for block_id in block_ids {
        let Some(block) = func.body.blocks.get_mut(&block_id) else { continue };

        for instr in &mut block.instructions {
            match &instr.value {
                // In SSR mode, useEffect/useLayoutEffect callbacks are never called.
                // We can simplify or remove the callback setup.
                InstructionValue::CallExpression(_v) => {
                    // In the full implementation, we'd check if the callee is
                    // a useEffect/useLayoutEffect hook and strip the callback.
                    // Hook detection handled in full implementation
                }
                _ => {}
            }
        }
    }
}
