/// Scope dependency utilities.
///
/// Port of `HIR/ScopeDependencyUtils.ts` from the React Compiler.
///
/// Utilities for working with reactive scope dependencies, including
/// merging, comparing, and minimizing dependency sets.
use crate::hir::{
    DependencyPath, DependencyPathEntry, IdentifierId, ReactiveScopeDependency,
    are_equal_paths, is_sub_path,
};
use rustc_hash::FxHashSet;

/// Merge two dependency sets, keeping only the minimal set of dependencies.
///
/// If dependency A is a sub-path of dependency B, only A is kept (since
/// A changing implies B's sub-path has also changed).
pub fn merge_dependencies(
    deps: &FxHashSet<ReactiveScopeDependency>,
) -> FxHashSet<ReactiveScopeDependency> {
    let deps_vec: Vec<&ReactiveScopeDependency> = deps.iter().collect();
    let mut result = FxHashSet::default();

    for (i, dep) in deps_vec.iter().enumerate() {
        let mut is_redundant = false;
        for (j, other) in deps_vec.iter().enumerate() {
            if i == j {
                continue;
            }
            // If this dep's identifier is the same and its path is a super-path of other's
            if dep.identifier_id == other.identifier_id
                && is_sub_path(&other.path, &dep.path)
                && !are_equal_paths(&other.path, &dep.path)
            {
                is_redundant = true;
                break;
            }
        }
        if !is_redundant {
            result.insert((*dep).clone());
        }
    }

    result
}
