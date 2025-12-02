use std::borrow::Cow;

use rustc_hash::{FxHashMap, FxHashSet};

use crate::ir_transform::sort_imports::{options::SortImportsOptions, source_line::SourceLine};

#[derive(Debug)]
pub struct SortableImport<'a> {
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

        // Stage 1: Separate ignored and non-ignored imports
        let (ignored_indices, sortable_indices): (Vec<usize>, Vec<usize>) =
            (0..imports_len).partition(|&idx| self[idx].is_ignored);

        // If all imports are ignored, no sorting needed
        if sortable_indices.is_empty() {
            return;
        }

        // Stage 2: Group non-ignored imports by `group_idx`
        let mut imports_by_group: FxHashMap<usize, Vec<usize>> = FxHashMap::default();
        for &idx in &sortable_indices {
            imports_by_group.entry(self[idx].group_idx).or_default().push(idx);
        }

        // Stage 3: Sort within each group and build sorted list
        // Need to process `groups` in order by `group_idx`
        let mut groups: Vec<_> = imports_by_group.iter_mut().collect();
        groups.sort_unstable_by_key(|(gidx, _)| *gidx);

        let mut sorted_indices = Vec::with_capacity(sortable_indices.len());
        for (_, group_indices) in groups {
            sort_within_group(group_indices, self, options);
            sorted_indices.extend_from_slice(group_indices);
        }

        // Stage 4: Build final permutation by inserting ignored imports at their original positions
        // If no ignored imports, we can skip the permutation step
        if ignored_indices.is_empty() {
            apply_permutation(self, &sorted_indices);
        } else {
            let ignored_set: FxHashSet<usize> = ignored_indices.into_iter().collect();
            let mut permutation = vec![0; imports_len];
            let mut sorted_iter = sorted_indices.into_iter();
            for (target_pos, perm) in permutation.iter_mut().enumerate() {
                if ignored_set.contains(&target_pos) {
                    // Ignored import stays at its original position
                    *perm = target_pos;
                } else if let Some(source_idx) = sorted_iter.next() {
                    *perm = source_idx;
                }
            }

            apply_permutation(self, &permutation);
        }

        debug_assert!(self.len() == imports_len, "Length must remain the same after sorting.");
    }
}

// ---

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
