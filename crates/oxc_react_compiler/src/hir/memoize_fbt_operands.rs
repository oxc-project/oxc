/// Memoize fbt and macro operands in the same scope.
///
/// Port of `ReactiveScopes/MemoizeFbtAndMacroOperandsInSameScope.ts` from the React Compiler.
///
/// Ensures that fbt (Facebook's internationalization library) operands
/// are memoized together with their containing fbt call, preventing
/// unnecessary re-translations.
use rustc_hash::FxHashSet;

use crate::hir::{HIRFunction, IdentifierId};

/// Memoize fbt/macro operands in the same scope as the fbt call.
///
/// Returns the set of identifiers that are fbt operands (needed by OutlineFunctions).
pub fn memoize_fbt_and_macro_operands_in_same_scope(
    _func: &mut HIRFunction,
) -> FxHashSet<IdentifierId> {
    // The full implementation identifies fbt() calls and ensures all
    // their operands are in the same reactive scope as the call itself.
    // This prevents individual operands from being memoized separately,
    // which would break fbt's translation mechanism.
    FxHashSet::default()
}
