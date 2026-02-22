/// Derive minimal dependencies for reactive scopes.
///
/// Port of `HIR/DeriveMinimalDependenciesHIR.ts` from the React Compiler.
///
/// This module provides a simplified tree-based approach for computing
/// minimal dependency sets. The full `ReactiveScopeDependencyTreeHIR` class
/// is ported in `propagate_scope_dependencies_hir.rs`; this module provides
/// standalone utility functions used by other passes.
///
/// Minimizes the set of dependencies for each reactive scope by removing
/// redundant dependencies. A dependency `a.b.c` is redundant if `a.b` is
/// also a dependency (since any change to `a.b.c` implies `a.b` changed too).
use rustc_hash::{FxHashMap, FxHashSet};

use super::hir_types::{
    DependencyPath, DependencyPathEntry, IdentifierId, ReactiveScopeDependency, is_sub_path,
};
use crate::compiler_error::SourceLocation;
use crate::hir::types::PropertyLiteral;

// =====================================================================================
// PropertyAccessType — how a property in the tree is accessed
// =====================================================================================

/// Access type for a property in the dependency tree.
///
/// We distinguish on two independent axes:
/// - Optional / Unconditional: whether the load is optional (within `?.`)
/// - Access / Dependency: whether we need to track change variables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PropertyAccessType {
    OptionalAccess,
    UnconditionalAccess,
    OptionalDependency,
    UnconditionalDependency,
}

impl PropertyAccessType {
    fn is_optional(self) -> bool {
        matches!(self, Self::OptionalAccess | Self::OptionalDependency)
    }

    fn is_dependency(self) -> bool {
        matches!(self, Self::OptionalDependency | Self::UnconditionalDependency)
    }

    fn merge(self, other: Self) -> Self {
        let result_unconditional = !(self.is_optional() && other.is_optional());
        let result_dependency = self.is_dependency() || other.is_dependency();
        match (result_unconditional, result_dependency) {
            (true, true) => Self::UnconditionalDependency,
            (true, false) => Self::UnconditionalAccess,
            (false, true) => Self::OptionalDependency,
            (false, false) => Self::OptionalAccess,
        }
    }
}

// =====================================================================================
// DependencyNode — a node in the dependency tree
// =====================================================================================

/// A node in the dependency tree.
struct DependencyNode {
    properties: FxHashMap<PropertyLiteral, DependencyNode>,
    access_type: PropertyAccessType,
    loc: SourceLocation,
}

// =====================================================================================
// HoistableNode — tracks safe property access paths
// =====================================================================================

/// Access type for hoistable property paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoistableAccessType {
    Optional,
    NonNull,
}

/// A node in the hoistable objects tree.
struct HoistableNode {
    properties: FxHashMap<PropertyLiteral, HoistableNode>,
    access_type: HoistableAccessType,
}

// =====================================================================================
// DependencyTree — tree for computing minimal dependency sets
// =====================================================================================

/// A tree structure for efficiently computing minimal dependency sets.
///
/// Port of `ReactiveScopeDependencyTreeHIR` from the TS source.
/// Each node represents a property access. The tree is built from the
/// full set of dependencies, then the minimal set is extracted by
/// taking only the shallowest dependency-marked paths.
pub struct DependencyTree {
    hoistable_objects: FxHashMap<IdentifierId, (HoistableNode, bool)>,
    deps: FxHashMap<IdentifierId, (DependencyNode, bool)>,
}

impl DependencyTree {
    /// Create a new dependency tree with the given set of hoistable object paths.
    pub fn new(hoistable_objects: impl IntoIterator<Item = ReactiveScopeDependency>) -> Self {
        let mut hoistable_map: FxHashMap<IdentifierId, (HoistableNode, bool)> =
            FxHashMap::default();

        for dep in hoistable_objects {
            let root = hoistable_map.entry(dep.identifier_id).or_insert_with(|| {
                let access_type = if dep.path.first().is_some_and(|e| e.optional) {
                    HoistableAccessType::Optional
                } else {
                    HoistableAccessType::NonNull
                };
                (HoistableNode { properties: FxHashMap::default(), access_type }, dep.reactive)
            });

            let mut cursor = &mut root.0;
            for (i, entry) in dep.path.iter().enumerate() {
                let access_type = if i + 1 < dep.path.len() && dep.path[i + 1].optional {
                    HoistableAccessType::Optional
                } else {
                    HoistableAccessType::NonNull
                };
                cursor = cursor.properties.entry(entry.property.clone()).or_insert_with(|| {
                    HoistableNode { properties: FxHashMap::default(), access_type }
                });
            }
        }

        Self { hoistable_objects: hoistable_map, deps: FxHashMap::default() }
    }

    /// Add a dependency, joining it with hoistable objects to determine
    /// the maximal safe-to-evaluate subpath.
    pub fn add_dependency(&mut self, dep: &ReactiveScopeDependency) {
        let root = self.deps.entry(dep.identifier_id).or_insert_with(|| {
            (
                DependencyNode {
                    properties: FxHashMap::default(),
                    access_type: PropertyAccessType::UnconditionalAccess,
                    loc: dep.loc,
                },
                dep.reactive,
            )
        });

        let mut dep_cursor = &raw mut root.0;
        let hoistable_root = self.hoistable_objects.get(&dep.identifier_id);
        let mut hoistable_cursor: Option<&HoistableNode> = hoistable_root.map(|(n, _)| n);

        for entry in &dep.path {
            let next_hoistable: Option<&HoistableNode>;
            let access_type;

            if entry.optional {
                next_hoistable = hoistable_cursor.and_then(|h| h.properties.get(&entry.property));

                access_type = if hoistable_cursor
                    .is_some_and(|h| h.access_type == HoistableAccessType::NonNull)
                {
                    PropertyAccessType::UnconditionalAccess
                } else {
                    PropertyAccessType::OptionalAccess
                };
            } else if hoistable_cursor
                .is_some_and(|h| h.access_type == HoistableAccessType::NonNull)
            {
                next_hoistable = hoistable_cursor.and_then(|h| h.properties.get(&entry.property));
                access_type = PropertyAccessType::UnconditionalAccess;
            } else {
                // Break to truncate the dependency at first non-hoistable entry
                break;
            }

            // SAFETY: `dep_cursor` points into `self.deps` map values. We only access disjoint
            // sub-nodes at each loop iteration, so no aliasing occurs.
            let dep_node = unsafe { &mut *dep_cursor };
            let child = dep_node.properties.entry(entry.property.clone()).or_insert_with(|| {
                DependencyNode { properties: FxHashMap::default(), access_type, loc: entry.loc }
            });
            child.access_type = child.access_type.merge(access_type);
            dep_cursor = std::ptr::from_mut::<DependencyNode>(child);
            hoistable_cursor = next_hoistable;
        }

        // Mark the final node as a dependency
        // SAFETY: `dep_cursor` is the final node pointer from the traversal above, with no
        // other mutable references to this node alive.
        let dep_node = unsafe { &mut *dep_cursor };
        dep_node.access_type = dep_node.access_type.merge(PropertyAccessType::OptionalDependency);
    }

    /// Derive the minimal set of dependencies from the tree.
    pub fn derive_minimal_dependencies(&self) -> Vec<ReactiveScopeDependency> {
        let mut results = Vec::new();
        for (&root_id, (root_node, reactive)) in &self.deps {
            collect_minimal_deps(root_node, *reactive, root_id, &[], &mut results);
        }
        results
    }
}

/// Recursively collect minimal dependencies from a subtree.
fn collect_minimal_deps(
    node: &DependencyNode,
    reactive: bool,
    root_id: IdentifierId,
    path: &[DependencyPathEntry],
    results: &mut Vec<ReactiveScopeDependency>,
) {
    if node.access_type.is_dependency() {
        results.push(ReactiveScopeDependency {
            identifier_id: root_id,
            reactive,
            path: path.to_vec(),
            loc: node.loc,
        });
    } else {
        for (child_name, child_node) in &node.properties {
            let mut new_path = path.to_vec();
            new_path.push(DependencyPathEntry {
                property: child_name.clone(),
                optional: child_node.access_type.is_optional(),
                loc: child_node.loc,
            });
            collect_minimal_deps(child_node, reactive, root_id, &new_path, results);
        }
    }
}

// =====================================================================================
// Standalone utility functions
// =====================================================================================

/// Derive the minimal set of dependencies from a full dependency set.
///
/// This removes dependencies that are sub-paths of other dependencies.
/// For example, if both `a.b` and `a.b.c` are dependencies, only `a.b`
/// is kept because any change to `a.b.c` must also change `a.b`.
pub fn derive_minimal_dependencies(
    dependencies: &[ReactiveScopeDependency],
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
        let common_len =
            a.iter().zip(b.iter()).take_while(|(ae, be)| ae.property == be.property).count();
        a[..common_len].to_vec()
    }
}
