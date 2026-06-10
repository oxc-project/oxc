use std::borrow::Cow;

use rustc_hash::FxHashMap;

use crate::ir_transform::sort_imports::{options::SortImportsOptions, source_line::SourceLine};

#[derive(Debug)]
pub struct SortableImport<'a> {
    /// Comments directly before this import (no empty line between).
    pub leading_lines: Vec<SourceLine<'a>>,
    pub import_line: SourceLine<'a>,
    // These are used for sorting and computed by `compute_import_metadata()`
    pub group_idx: usize,
    pub normalized_source: Cow<'a, str>,
    pub is_side_effect: bool,
    pub is_ignored: bool,
}

// ---

pub trait SortSortableImports {
    fn sort(&mut self, options: &SortImportsOptions);
}

impl SortSortableImports for Vec<SortableImport<'_>> {
    fn sort(&mut self, options: &SortImportsOptions) {
        let imports_len = self.len();

        // Perform sorting only if needed
        if imports_len < 2 {
            return;
        }

        // Build a global permutation while sorting each contiguous non-ignored partition independently.
        // This prevents sortable imports from crossing ignored imports (e.g. side effects with
        // `sortSideEffects: false`), while keeping ignored imports fixed at their original positions.
        let mut permutation: Vec<usize> = (0..imports_len).collect();
        let mut partition_start: Option<usize> = None;

        for idx in 0..=imports_len {
            let is_partition_end = idx == imports_len || self[idx].is_ignored;
            if is_partition_end {
                if let Some(start) = partition_start.take() {
                    let sorted_indices =
                        sort_indices_by_group_for_partition(start, idx, self, options);
                    for (target_pos, source_idx) in (start..idx).zip(sorted_indices) {
                        permutation[target_pos] = source_idx;
                    }
                }
            } else if partition_start.is_none() {
                partition_start = Some(idx);
            }
        }

        apply_permutation(self, &permutation);
        debug_assert!(self.len() == imports_len, "Length must remain the same after sorting.");
    }
}

// ---

/// Sort one contiguous non-ignored partition.
///
/// Returns source indices in the target order for this partition's slots.
fn sort_indices_by_group_for_partition(
    start: usize,
    end: usize,
    imports: &[SortableImport],
    options: &SortImportsOptions,
) -> Vec<usize> {
    let mut imports_by_group: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
    for (idx, import) in imports.iter().enumerate().take(end).skip(start) {
        imports_by_group.entry(import.group_idx).or_default().push(idx);
    }

    let mut groups: Vec<_> = imports_by_group.into_iter().collect();
    groups.sort_unstable_by_key(|(gidx, _)| *gidx);

    let mut sorted_indices = Vec::with_capacity(end - start);
    for (_, mut group_indices) in groups {
        sort_within_group(&mut group_indices, imports, options);
        sorted_indices.extend(group_indices);
    }

    sorted_indices
}

/// Sort imports within a single group, respecting side-effect preservation rules.
///
/// - If `sort_side_effects: true`: sorts all imports by source
/// - If `sort_side_effects: false`: preserves relative order of side-effect imports
///   - While sorting non-side-effect imports, then merges them back together
fn sort_within_group(
    indices: &mut [usize],
    imports: &[SortableImport],
    options: &SortImportsOptions,
) {
    if indices.len() < 2 {
        return;
    }

    debug_assert!(
        indices.iter().all(|&idx| !imports[idx].is_ignored),
        "All imports in `indices` must be non-ignored"
    );

    // If `sort_side_effects: true`, sort all imports together
    if options.sort_side_effects {
        sort_indices_by_source(indices, imports, options);
        return;
    }

    // Otherwise, preserve side-effect order while sorting non-side-effects
    let mut side_effect_indices = vec![];
    let mut non_side_effect_indices = vec![];
    for (pos, &idx) in indices.iter().enumerate() {
        if imports[idx].is_side_effect {
            side_effect_indices.push((pos, idx));
        } else {
            non_side_effect_indices.push(idx);
        }
    }

    // Sort only non-side-effect imports
    sort_indices_by_source(&mut non_side_effect_indices, imports, options);

    // Merge side-effects back at their original relative positions
    let mut result = Vec::with_capacity(indices.len());
    let mut side_effect_iter = side_effect_indices.into_iter();
    let mut non_side_effect_iter = non_side_effect_indices.into_iter();
    let mut next_side_effect = side_effect_iter.next();

    for pos in 0..indices.len() {
        match next_side_effect {
            Some((se_pos, se_idx)) if se_pos == pos => {
                result.push(se_idx);
                next_side_effect = side_effect_iter.next();
            }
            _ => {
                if let Some(non_se_idx) = non_side_effect_iter.next() {
                    result.push(non_se_idx);
                }
            }
        }
    }

    indices.copy_from_slice(&result);
}

/// Sort indices by their normalized source.
fn sort_indices_by_source(
    indices: &mut [usize],
    imports: &[SortableImport],
    options: &SortImportsOptions,
) {
    indices.sort_by(|&a, &b| {
        natord::compare(&imports[a].normalized_source, &imports[b].normalized_source)
    });

    if options.order.is_desc() {
        indices.reverse();
    }
}

// ---

/// Apply permutation in-place using cycle decomposition.
fn apply_permutation(imports: &mut [SortableImport], permutation: &[usize]) {
    let mut visited = vec![false; imports.len()];
    for idx in 0..imports.len() {
        // Already visited or already in the correct position
        if visited[idx] || permutation[idx] == idx {
            continue;
        }
        // Follow the cycle
        let mut current = idx;
        loop {
            let next = permutation[current];
            visited[current] = true;
            if next == idx {
                break;
            }
            imports.swap(current, next);
            current = next;
        }
    }
}
