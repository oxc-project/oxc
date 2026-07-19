//! Insertion-ordered map and set backed by a `Vec` of entries plus an
//! `FxHashMap` index for O(1) key lookup.
//!
//! These are drop-in replacements for the `indexmap`-based [`FxIndexMap`] /
//! [`FxIndexSet`] used by the HIR (`HIR.blocks`, `BasicBlock.preds`,
//! `Phi.operands`). They exist because `indexmap` has no arena-allocator variant,
//! whereas `Vec` and hash maps do (`oxc_allocator::Vec` / `oxc_allocator::HashMap`).
//! Rebuilding the ordered map from those primitives is the first step toward
//! moving the HIR into an arena; the container semantics (insertion order + O(1)
//! lookup) match `IndexMap`/`IndexSet` so behavior is unchanged.
//!
//! [`FxIndexMap`]: crate::react_compiler_utils::FxIndexMap
//! [`FxIndexSet`]: crate::react_compiler_utils::FxIndexSet

use std::fmt;
use std::hash::Hash;

use rustc_hash::FxHashMap;

/// Insertion-ordered map with O(1) keyed lookup. Mirrors the subset of the
/// `IndexMap` API used by the HIR. Keys are `Copy` (all current keys are ids).
#[derive(Clone)]
pub struct OrderedMap<K, V> {
    /// Entries in insertion order.
    entries: Vec<(K, V)>,
    /// Maps a key to its position in `entries`.
    index: FxHashMap<K, usize>,
}

impl<K, V> Default for OrderedMap<K, V> {
    fn default() -> Self {
        Self { entries: Vec::new(), index: FxHashMap::default() }
    }
}

impl<K: Copy + Eq + Hash, V> OrderedMap<K, V> {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
    }

    pub fn contains_key(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }

    /// Position of `key` in insertion order, if present.
    pub fn get_index_of(&self, key: &K) -> Option<usize> {
        self.index.get(key).copied()
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.index.get(key).map(|&i| &self.entries[i].1)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        match self.index.get(key) {
            Some(&i) => Some(&mut self.entries[i].1),
            None => None,
        }
    }

    /// Insert `value` for `key`. If the key already exists its position is kept
    /// and the previous value is returned; otherwise the entry is appended.
    /// Matches `IndexMap::insert`.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if let Some(&i) = self.index.get(&key) {
            Some(std::mem::replace(&mut self.entries[i].1, value))
        } else {
            let i = self.entries.len();
            self.entries.push((key, value));
            self.index.insert(key, i);
            None
        }
    }

    /// Remove `key`, shifting later entries down to preserve order (O(n)).
    /// Matches `IndexMap::shift_remove`.
    pub fn shift_remove(&mut self, key: &K) -> Option<V> {
        let i = self.index.remove(key)?;
        let (_, value) = self.entries.remove(i);
        // Entries after `i` moved down by one; fix their recorded positions.
        for j in i..self.entries.len() {
            let k = self.entries[j].0;
            self.index.insert(k, j);
        }
        Some(value)
    }

    /// Keep only entries for which `keep` returns true, preserving order.
    /// Matches `IndexMap::retain`.
    pub fn retain(&mut self, mut keep: impl FnMut(&K, &mut V) -> bool) {
        self.entries.retain_mut(|(k, v)| keep(k, v));
        self.index.clear();
        for (i, (k, _)) in self.entries.iter().enumerate() {
            self.index.insert(*k, i);
        }
    }

    /// Remove and yield all entries (the only range used is `..`).
    pub fn drain(&mut self, range: std::ops::RangeFull) -> std::vec::Drain<'_, (K, V)> {
        self.index.clear();
        self.entries.drain(range)
    }

    pub fn keys(&self) -> impl DoubleEndedIterator<Item = &K> {
        self.entries.iter().map(|(k, _)| k)
    }

    pub fn values(&self) -> impl DoubleEndedIterator<Item = &V> {
        self.entries.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut V> {
        self.entries.iter_mut().map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = (&K, &V)> {
        self.entries.iter().map(|(k, v)| (k, v))
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = (&K, &mut V)> {
        self.entries.iter_mut().map(|(k, v)| (&*k, v))
    }
}

impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for OrderedMap<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.entries.iter().map(|(k, v)| (k, v))).finish()
    }
}

/// Insertion-ordered set with O(1) membership. Mirrors the subset of the
/// `IndexSet` API used by the HIR.
#[derive(Clone)]
pub struct OrderedSet<K> {
    entries: Vec<K>,
    index: FxHashMap<K, usize>,
}

impl<K> Default for OrderedSet<K> {
    fn default() -> Self {
        Self { entries: Vec::new(), index: FxHashMap::default() }
    }
}

impl<K: Copy + Eq + Hash> OrderedSet<K> {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.index.clear();
    }

    pub fn contains(&self, key: &K) -> bool {
        self.index.contains_key(key)
    }

    /// Insert `key`; returns true if it was newly added. Matches `IndexSet::insert`.
    pub fn insert(&mut self, key: K) -> bool {
        if self.index.contains_key(&key) {
            return false;
        }
        let i = self.entries.len();
        self.entries.push(key);
        self.index.insert(key, i);
        true
    }

    /// Remove `key`, shifting later entries down to preserve order (O(n)).
    /// Matches `IndexSet::shift_remove`.
    pub fn shift_remove(&mut self, key: &K) -> bool {
        let Some(i) = self.index.remove(key) else {
            return false;
        };
        self.entries.remove(i);
        for j in i..self.entries.len() {
            let k = self.entries[j];
            self.index.insert(k, j);
        }
        true
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &K> {
        self.entries.iter()
    }
}

impl<K: fmt::Debug> fmt::Debug for OrderedSet<K> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_set().entries(self.entries.iter()).finish()
    }
}

/// Borrowing iterator over an [`OrderedMap`], yielding `(&K, &V)` in insertion order.
pub struct Iter<'a, K, V>(std::slice::Iter<'a, (K, V)>);

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (k, v))
    }
}

/// Mutable-borrowing iterator over an [`OrderedMap`], yielding `(&K, &mut V)`.
pub struct IterMut<'a, K, V>(std::slice::IterMut<'a, (K, V)>);

impl<'a, K, V> Iterator for IterMut<'a, K, V> {
    type Item = (&'a K, &'a mut V);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(k, v)| (&*k, v))
    }
}

impl<K, V> IntoIterator for OrderedMap<K, V> {
    type Item = (K, V);
    type IntoIter = std::vec::IntoIter<(K, V)>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a, K, V> IntoIterator for &'a OrderedMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter(self.entries.iter())
    }
}

impl<'a, K, V> IntoIterator for &'a mut OrderedMap<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        IterMut(self.entries.iter_mut())
    }
}

impl<K: Copy + Eq + Hash, V> std::ops::Index<&K> for OrderedMap<K, V> {
    type Output = V;
    fn index(&self, key: &K) -> &V {
        self.get(key).expect("OrderedMap: no entry found for key")
    }
}

impl<K> IntoIterator for OrderedSet<K> {
    type Item = K;
    type IntoIter = std::vec::IntoIter<K>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_iter()
    }
}

impl<'a, K> IntoIterator for &'a OrderedSet<K> {
    type Item = &'a K;
    type IntoIter = std::slice::Iter<'a, K>;
    fn into_iter(self) -> Self::IntoIter {
        self.entries.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::{OrderedMap, OrderedSet};

    #[test]
    fn map_insert_order_and_lookup() {
        let mut m: OrderedMap<u32, &str> = OrderedMap::default();
        assert_eq!(m.insert(3, "a"), None);
        assert_eq!(m.insert(1, "b"), None);
        assert_eq!(m.insert(2, "c"), None);
        // Re-insert keeps position, returns old value.
        assert_eq!(m.insert(1, "B"), Some("b"));
        assert_eq!(m.keys().copied().collect::<Vec<_>>(), vec![3, 1, 2]);
        assert_eq!(m.get(&1), Some(&"B"));
        assert_eq!(m.get_index_of(&2), Some(2));
        assert_eq!(m.get_index_of(&9), None);
        assert_eq!(m.len(), 3);
    }

    #[test]
    fn map_shift_remove_preserves_order_and_reindexes() {
        let mut m: OrderedMap<u32, u32> = OrderedMap::default();
        for k in [10, 20, 30, 40] {
            m.insert(k, k * 10);
        }
        assert_eq!(m.shift_remove(&20), Some(200));
        assert_eq!(m.shift_remove(&99), None);
        assert_eq!(m.keys().copied().collect::<Vec<_>>(), vec![10, 30, 40]);
        // Every surviving key still resolves to the right value after reindexing.
        assert_eq!(m.get(&10), Some(&100));
        assert_eq!(m.get(&30), Some(&300));
        assert_eq!(m.get(&40), Some(&400));
        assert_eq!(m.get_index_of(&40), Some(2));
    }

    #[test]
    fn map_retain_and_drain() {
        let mut m: OrderedMap<u32, u32> = OrderedMap::default();
        for k in [1, 2, 3, 4, 5] {
            m.insert(k, k);
        }
        m.retain(|k, _| k % 2 == 1);
        assert_eq!(
            m.iter().map(|(k, v)| (*k, *v)).collect::<Vec<_>>(),
            vec![(1, 1), (3, 3), (5, 5)]
        );
        assert_eq!(m.get_index_of(&5), Some(2));
        let drained: Vec<_> = m.drain(..).collect();
        assert_eq!(drained, vec![(1, 1), (3, 3), (5, 5)]);
        assert!(m.is_empty());
        assert_eq!(m.get(&1), None);
    }

    #[test]
    fn set_insert_contains_shift_remove() {
        let mut s: OrderedSet<u32> = OrderedSet::default();
        assert!(s.insert(5));
        assert!(s.insert(1));
        assert!(!s.insert(5)); // duplicate
        assert!(s.contains(&1));
        assert_eq!(s.iter().copied().collect::<Vec<_>>(), vec![5, 1]);
        assert!(s.shift_remove(&5));
        assert!(!s.shift_remove(&5));
        assert_eq!(s.iter().copied().collect::<Vec<_>>(), vec![1]);
        assert!(!s.contains(&5));
    }
}
