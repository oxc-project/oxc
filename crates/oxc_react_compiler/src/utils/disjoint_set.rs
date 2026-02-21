/// Represents items which form disjoint sets.
///
/// Port of `Utils/DisjointSet.ts` from the React Compiler.
///
/// This is a union-find data structure with path compression.
/// Items of type `T` must implement `Hash + Eq + Clone`.
use std::hash::Hash;

use rustc_hash::{FxHashMap, FxHashSet};

pub struct DisjointSet<T: Hash + Eq + Clone> {
    entries: FxHashMap<T, T>,
}

impl<T: Hash + Eq + Clone> Default for DisjointSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Hash + Eq + Clone> DisjointSet<T> {
    pub fn new() -> Self {
        Self { entries: FxHashMap::default() }
    }

    /// Updates the graph to reflect that the given `items` form a set,
    /// linking any previous sets that the items were part of into a single set.
    ///
    /// # Panics
    ///
    /// Panics if `items` is empty.
    pub fn union(&mut self, items: &[T]) {
        assert!(!items.is_empty(), "Expected set to be non-empty");

        let first = &items[0];

        // Determine an arbitrary "root" for this set: if the first
        // item already has a root then use that, otherwise the first item
        // will be the new root.
        let root = if let Some(r) = self.find(first) {
            r
        } else {
            self.entries.insert(first.clone(), first.clone());
            first.clone()
        };

        // Update remaining items (which may already be part of other sets)
        for item in &items[1..] {
            let item_parent = self.entries.get(item).cloned();
            match item_parent {
                None => {
                    // New item, no existing set to update
                    self.entries.insert(item.clone(), root.clone());
                }
                Some(ref parent) if parent == &root => {
                    // Already points to root
                }
                Some(_) => {
                    let mut current = item.clone();
                    loop {
                        let item_parent =
                            self.entries.get(&current).cloned().expect("entry must exist");
                        if item_parent == root {
                            break;
                        }
                        self.entries.insert(current.clone(), root.clone());
                        current = item_parent;
                    }
                }
            }
        }
    }

    /// Finds the set to which the given `item` is associated, if `item`
    /// is present in this set. If item is not present, returns `None`.
    ///
    /// Note that the returned value may be any item in the set to which the input
    /// belongs: the only guarantee is that all items in a set will return the same
    /// value in between calls to `union()`.
    ///
    /// # Panics
    /// Panics if internal data is inconsistent (parent entry missing).
    pub fn find(&mut self, item: &T) -> Option<T> {
        if !self.entries.contains_key(item) {
            return None;
        }
        let parent = self.entries.get(item).cloned().expect("entry must exist");
        if parent == *item {
            // This is the root element
            return Some(parent);
        }
        // Recurse to find the root (caching all elements along the path to the root)
        let root = self.find(&parent).expect("parent must exist in entries");
        // Cache the element itself (path compression)
        self.entries.insert(item.clone(), root.clone());
        Some(root)
    }

    /// Returns `true` if the given `item` is present in this set.
    pub fn has(&self, item: &T) -> bool {
        self.entries.contains_key(item)
    }

    /// Forces the set into canonical form, ie with all items pointing directly to
    /// their root, and returns a map representing the mapping of items to their roots.
    ///
    /// # Panics
    /// Panics if internal data is inconsistent.
    pub fn canonicalize(&mut self) -> FxHashMap<T, T> {
        let keys: Vec<T> = self.entries.keys().cloned().collect();
        let mut result = FxHashMap::default();
        for item in keys {
            let root = self.find(&item).expect("item must exist in entries");
            result.insert(item, root);
        }
        result
    }

    /// Calls the provided callback once for each item in the disjoint set,
    /// passing the item and the group to which it belongs.
    ///
    /// # Panics
    /// Panics if internal data is inconsistent.
    pub fn for_each(&mut self, mut f: impl FnMut(&T, &T)) {
        let keys: Vec<T> = self.entries.keys().cloned().collect();
        for item in &keys {
            let group = self.find(item).expect("item must exist in entries");
            f(item, &group);
        }
    }

    /// Builds non-overlapping sets from the disjoint set.
    ///
    /// # Panics
    /// Panics if internal data is inconsistent.
    pub fn build_sets(&mut self) -> Vec<FxHashSet<T>> {
        let mut ids: FxHashMap<T, usize> = FxHashMap::default();
        let mut sets: FxHashMap<usize, FxHashSet<T>> = FxHashMap::default();

        let keys: Vec<T> = self.entries.keys().cloned().collect();
        for item in keys {
            let group = self.find(&item).expect("item must exist in entries");

            let id = {
                let next_id = ids.len();
                *ids.entry(group).or_insert(next_id)
            };

            sets.entry(id).or_default().insert(item);
        }

        sets.into_values().collect()
    }

    /// Returns the number of items in the disjoint set.
    pub fn size(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    struct TestIdentifier {
        id: u32,
        name: String,
    }

    static NEXT_ID: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

    fn reset_ids() {
        NEXT_ID.store(0, std::sync::atomic::Ordering::SeqCst);
    }

    fn make_identifier(name: &str) -> TestIdentifier {
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        TestIdentifier { id, name: name.to_string() }
    }

    fn make_identifiers(names: &[&str]) -> Vec<TestIdentifier> {
        names.iter().map(|name| make_identifier(name)).collect()
    }

    #[test]
    fn find_finds_correct_group() {
        reset_ids();
        let mut identifiers = DisjointSet::new();
        let ids = make_identifiers(&["x", "y", "z"]);
        let (x, y, z) = (ids[0].clone(), ids[1].clone(), ids[2].clone());

        identifiers.union(&[x.clone()]);
        identifiers.union(&[y.clone(), x.clone()]);

        assert_eq!(identifiers.find(&x), Some(y.clone()));
        assert_eq!(identifiers.find(&y), Some(y.clone()));
        assert_eq!(identifiers.find(&z), None);
    }

    #[test]
    fn size_returns_0_when_empty() {
        let identifiers: DisjointSet<TestIdentifier> = DisjointSet::new();
        assert_eq!(identifiers.size(), 0);
    }

    #[test]
    fn size_returns_correct_size() {
        reset_ids();
        let mut identifiers = DisjointSet::new();
        let ids = make_identifiers(&["x", "y", "z"]);
        let (x, y) = (ids[0].clone(), ids[1].clone());

        identifiers.union(&[x.clone()]);
        identifiers.union(&[y.clone(), x.clone()]);

        assert_eq!(identifiers.size(), 2);
    }

    #[test]
    fn build_sets_returns_non_overlapping_sets() {
        reset_ids();
        let mut identifiers = DisjointSet::new();
        let ids = make_identifiers(&["a", "b", "c", "x", "y", "z"]);
        let (a, b, c, x, y, z) = (
            ids[0].clone(),
            ids[1].clone(),
            ids[2].clone(),
            ids[3].clone(),
            ids[4].clone(),
            ids[5].clone(),
        );

        identifiers.union(&[a.clone()]);
        identifiers.union(&[b.clone(), a.clone()]);
        identifiers.union(&[c.clone(), b.clone()]);

        identifiers.union(&[x.clone()]);
        identifiers.union(&[y.clone(), x.clone()]);
        identifiers.union(&[z.clone(), y.clone()]);
        identifiers.union(&[x.clone(), z.clone()]);

        let sets = identifiers.build_sets();
        assert_eq!(sets.len(), 2);

        // Verify one set contains {a, b, c} and the other {x, y, z}
        let set_abc: FxHashSet<TestIdentifier> =
            [a.clone(), b.clone(), c.clone()].into_iter().collect();
        let set_xyz: FxHashSet<TestIdentifier> =
            [x.clone(), y.clone(), z.clone()].into_iter().collect();

        let found_abc = sets.iter().any(|s| *s == set_abc);
        let found_xyz = sets.iter().any(|s| *s == set_xyz);
        assert!(found_abc, "Expected to find set {{a, b, c}}");
        assert!(found_xyz, "Expected to find set {{x, y, z}}");
    }

    /// Regression test for issue #933
    #[test]
    fn for_each_no_infinite_loop_with_cycles() {
        reset_ids();
        let mut identifiers = DisjointSet::new();
        let ids = make_identifiers(&["x", "y", "z"]);
        let (x, y, z) = (ids[0].clone(), ids[1].clone(), ids[2].clone());

        identifiers.union(&[x.clone()]);
        identifiers.union(&[y.clone(), x.clone()]);
        identifiers.union(&[z.clone(), y.clone()]);
        identifiers.union(&[x.clone(), z.clone()]);

        identifiers.for_each(|_, group| {
            assert_eq!(group, &z);
        });
    }
}
