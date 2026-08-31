//! Order-preserving permutation helpers.
//!
//! Vocabulary: an index is *pinned* when it must stay exactly where it is while the other
//! indices flow around it (an ignored side-effect import, a suppressed member, …).

use std::{cmp::Ordering, collections::BTreeMap};

/// Stable-sort the non-pinned entries of `indices` by `compare`, leaving pinned entries in place.
///
/// `indices` holds element indices; `is_pinned` and `compare` are called with those indices.
pub fn sort_with_pinned(
    indices: &mut [usize],
    is_pinned: impl Fn(usize) -> bool,
    mut compare: impl FnMut(usize, usize) -> Ordering,
) {
    let mut movable: Vec<usize> = indices.iter().copied().filter(|&i| !is_pinned(i)).collect();
    if movable.len() < 2 {
        return;
    }
    movable.sort_by(|&a, &b| compare(a, b));

    let mut movable = movable.into_iter();
    for slot in indices.iter_mut() {
        if !is_pinned(*slot) {
            *slot = movable.next().expect("one movable index per non-pinned slot");
        }
    }
}

/// Bucket `0..len` by group index, sort each bucket with `sort_group`, concatenate the buckets
/// in ascending group order, then hand the result out over the non-pinned slots so that pinned
/// indices stay at their own position.
///
/// Returns `permutation` with `permutation[target] = source`.
pub fn group_sort(
    len: usize,
    group_of: impl Fn(usize) -> usize,
    is_pinned: impl Fn(usize) -> bool,
    mut sort_group: impl FnMut(&mut [usize]),
) -> Vec<usize> {
    // `BTreeMap` iterates in ascending group order.
    let mut by_group: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for idx in 0..len {
        if !is_pinned(idx) {
            by_group.entry(group_of(idx)).or_default().push(idx);
        }
    }

    let mut sorted = Vec::with_capacity(len);
    for mut bucket in by_group.into_values() {
        sort_group(&mut bucket);
        sorted.extend(bucket);
    }

    let mut sorted = sorted.into_iter();
    (0..len)
        .map(|target| {
            if is_pinned(target) {
                target
            } else {
                sorted.next().expect("one sorted index per non-pinned slot")
            }
        })
        .collect()
}

/// Apply `permutation[target] = source` in place using cycle decomposition.
pub fn apply_permutation<T>(items: &mut [T], permutation: &[usize]) {
    debug_assert_eq!(items.len(), permutation.len());
    let mut visited = vec![false; items.len()];
    for idx in 0..items.len() {
        if visited[idx] || permutation[idx] == idx {
            continue;
        }
        let mut current = idx;
        loop {
            let next = permutation[current];
            visited[current] = true;
            if next == idx {
                break;
            }
            items.swap(current, next);
            current = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_permutation_three_cycle() {
        let mut items = vec!["a", "b", "c"];
        apply_permutation(&mut items, &[1, 2, 0]);
        assert_eq!(items, vec!["b", "c", "a"]);
    }

    #[test]
    fn apply_permutation_identity_is_noop() {
        let mut items = vec![1, 2, 3];
        apply_permutation(&mut items, &[0, 1, 2]);
        assert_eq!(items, vec![1, 2, 3]);
    }

    #[test]
    fn sort_with_pinned_keeps_pinned_slots() {
        let names = ["d", "PIN", "a", "c"];
        let mut indices = vec![0, 1, 2, 3];
        sort_with_pinned(&mut indices, |i| i == 1, |a, b| names[a].cmp(names[b]));
        assert_eq!(indices, vec![2, 1, 3, 0]);
    }

    #[test]
    fn sort_with_pinned_is_stable() {
        let names = ["b", "a", "a"];
        let mut indices = vec![0, 1, 2];
        sort_with_pinned(&mut indices, |_| false, |a, b| names[a].cmp(names[b]));
        assert_eq!(indices, vec![1, 2, 0]);
    }

    #[test]
    fn sort_with_pinned_with_one_or_zero_movable_is_noop() {
        let mut indices = vec![0, 1, 2];
        sort_with_pinned(&mut indices, |i| i != 1, |a, b| a.cmp(&b));
        assert_eq!(indices, vec![0, 1, 2]);

        let mut indices = vec![0, 1, 2];
        sort_with_pinned(&mut indices, |_| true, |a, b| a.cmp(&b));
        assert_eq!(indices, vec![0, 1, 2]);
    }

    #[test]
    fn group_sort_orders_groups_then_within() {
        let names = ["b", "y", "a", "x", "z"];
        let groups = [1, 0, 1, 0, 0];
        let perm = group_sort(
            names.len(),
            |i| groups[i],
            |_| false,
            |bucket| bucket.sort_by(|&a, &b| names[a].cmp(names[b])),
        );
        assert_eq!(perm, vec![3, 1, 4, 2, 0]);
    }

    #[test]
    fn group_sort_keeps_pinned_at_their_index() {
        let names = ["b", "PIN", "a"];
        let groups = [1, 0, 1];
        let perm = group_sort(
            names.len(),
            |i| groups[i],
            |i| i == 1,
            |bucket| bucket.sort_by(|&a, &b| names[a].cmp(names[b])),
        );
        assert_eq!(perm, vec![2, 1, 0]);
    }

    #[test]
    fn group_sort_of_sorted_input_is_identity() {
        let names = ["a", "b", "c"];
        let perm = group_sort(
            3,
            |_| 0,
            |_| false,
            |bucket| {
                bucket.sort_by(|&a, &b| names[a].cmp(names[b]));
            },
        );
        assert_eq!(perm, vec![0, 1, 2]);
    }

    #[test]
    fn group_sort_all_pinned_is_identity() {
        let perm = group_sort(3, |_| 0, |_| true, |_| unreachable!("no bucket to sort"));
        assert_eq!(perm, vec![0, 1, 2]);
    }

    #[test]
    fn apply_permutation_two_disjoint_cycles() {
        let mut items = vec!["a", "b", "c", "d"];
        apply_permutation(&mut items, &[1, 0, 3, 2]);
        assert_eq!(items, vec!["b", "a", "d", "c"]);
    }
}
