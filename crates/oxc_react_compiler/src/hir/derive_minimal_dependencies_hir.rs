/// Derive minimal dependencies for reactive scopes.
///
/// Port of `HIR/DeriveMinimalDependenciesHIR.ts` from the React Compiler.
///
/// Minimizes the set of dependencies for each reactive scope by removing
/// redundant dependencies. A dependency `a.b.c` is redundant if `a.b` is
/// also a dependency (since any change to `a.b.c` implies `a.b` changed too).
use rustc_hash::FxHashSet;

use super::hir_types::{
    DependencyPath, IdentifierId, ReactiveScopeDependency, is_sub_path,
};

/// A tree structure for efficiently computing minimal dependency sets.
///
/// Each node represents a property access. The tree is built from the
/// full set of dependencies, then the minimal set is extracted by
/// taking only the shallowest paths.
/// Internal tree structure for dependency minimization (used in full impl).
pub struct DependencyTree {
    /// Root-level dependency identifiers.
    pub root_deps: FxHashSet<IdentifierId>,
}

/// Derive the minimal set of dependencies from a full dependency set.
///
/// This removes dependencies that are sub-paths of other dependencies.
/// For example, if both `a.b` and `a.b.c` are dependencies, only `a.b`
/// is kept because any change to `a.b.c` must also change `a.b`.
pub fn derive_minimal_dependencies(
    dependencies: &FxHashSet<ReactiveScopeDependency>,
) -> FxHashSet<ReactiveScopeDependency> {
    let mut minimal = FxHashSet::default();
    let deps: Vec<&ReactiveScopeDependency> = dependencies.iter().collect();

    for dep in &deps {
        let is_redundant = deps.iter().any(|other| {
            // Check if `dep` is a longer path of `other` (other is a prefix of dep)
            dep.identifier_id == other.identifier_id
                && other.path.len() < dep.path.len()
                && is_sub_path(&other.path, &dep.path)
        });

        if !is_redundant {
            minimal.insert((*dep).clone());
        }
    }

    minimal
}

/// Merge two dependency paths, keeping the shorter (more general) one.
pub fn merge_dependency_paths(a: &DependencyPath, b: &DependencyPath) -> DependencyPath {
    if is_sub_path(a, b) {
        a.clone()
    } else if is_sub_path(b, a) {
        b.clone()
    } else {
        // Paths diverge — keep the common prefix
        let common_len = a
            .iter()
            .zip(b.iter())
            .take_while(|(ae, be)| ae.property == be.property)
            .count();
        a[..common_len].to_vec()
    }
}
