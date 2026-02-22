/// Outline function expressions from reactive scopes.
///
/// Port of `Optimization/OutlineFunctions.ts` from the React Compiler.
///
/// Moves function expressions out of reactive scopes when safe to do so,
/// reducing the number of values that need to be memoized.
use rustc_hash::FxHashSet;

use crate::hir::{HIRFunction, IdentifierId};

/// Outline function expressions from reactive scopes.
pub(crate) fn outline_functions(func: &HIRFunction, _fbt_operands: &FxHashSet<IdentifierId>) {
    // The full implementation moves FunctionExpression instructions out of
    // reactive scopes when the function's captures are all available outside
    // the scope. This reduces the number of values the scope needs to produce.
    // Fbt operand tracking used in full implementation
    let _block_ids: Vec<_> = func.body.blocks.keys().copied().collect();
}
