use cow_utils::CowUtils;

use crate::{formatter::format_element::FormatElement, options};

use super::source_line::{ImportLine, SourceLine};

#[derive(Debug)]
pub struct ImportUnits(pub Vec<SortableImport>);

impl IntoIterator for ImportUnits {
    type Item = SortableImport;
    type IntoIter = std::vec::IntoIter<SortableImport>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl ImportUnits {
    // TODO: Sort based on `options.groups`, `options.type`, etc...
    // TODO: Consider `special_characters`, removing `?raw`, etc...
    pub fn sort_imports(&mut self, elements: &[FormatElement], options: options::SortImports) {
        let imports_len = self.0.len();

        // Perform sorting only if needed
        if imports_len < 2 {
            return;
        }

        // Separate imports into:
        // - sortable: indices of imports that should be sorted
        // - fixed: indices of imports that should be ignored
        //   - e.g. side-effect imports when `sort_side_effects: false`, with ignore comments, etc...
        let mut sortable_indices = vec![];
        let mut fixed_indices = vec![];
        for (idx, si) in self.0.iter().enumerate() {
            if si.is_ignored(options) {
                fixed_indices.push(idx);
            } else {
                sortable_indices.push(idx);
            }
        }

        // Sort indices by comparing their corresponding import sources
        sortable_indices.sort_by(|&a, &b| {
            let source_a = self.0[a].get_source(elements);
            let source_b = self.0[b].get_source(elements);

            let ord = if options.ignore_case {
                source_a.cow_to_lowercase().cmp(&source_b.cow_to_lowercase())
            } else {
                source_a.cmp(source_b)
            };

            if options.order.is_desc() { ord.reverse() } else { ord }
        });

        // Create a permutation map
        let mut permutation = vec![0; imports_len];
        let mut sortable_iter = sortable_indices.into_iter();
        for (idx, perm) in permutation.iter_mut().enumerate() {
            // NOTE: This is O(n), but side-effect imports are usually few
            if fixed_indices.contains(&idx) {
                *perm = idx;
            } else if let Some(sorted_idx) = sortable_iter.next() {
                *perm = sorted_idx;
            }
        }
        debug_assert!(
            permutation.iter().copied().collect::<rustc_hash::FxHashSet<_>>().len() == imports_len,
            "`permutation` must be a valid permutation, all indices must be unique."
        );

        // Apply permutation in-place using cycle decomposition
        let mut visited = vec![false; imports_len];
        for idx in 0..imports_len {
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
                self.0.swap(current, next);
                current = next;
            }
        }
        debug_assert!(self.0.len() == imports_len, "Length must remain the same after sorting.");
    }
}

#[derive(Debug, Clone)]
pub struct SortableImport {
    pub leading_lines: Vec<SourceLine>,
    pub import_line: SourceLine,
}

impl SortableImport {
    pub fn new(leading_lines: Vec<SourceLine>, import_line: SourceLine) -> Self {
        Self { leading_lines, import_line }
    }

    /// Get the import source string for sorting.
    /// e.g. `"./foo"`, `"react"`, etc...
    /// This includes quotes, but will not affect sorting.
    /// Since they are already normalized by the formatter.
    pub fn get_source<'a>(&self, elements: &'a [FormatElement]) -> &'a str {
        let SourceLine::Import(ImportLine { source_idx, .. }) = &self.import_line else {
            unreachable!("`import_line` must be of type `SourceLine::Import`.");
        };
        match &elements[*source_idx] {
            FormatElement::LocatedTokenText { slice, .. } => slice,
            FormatElement::DynamicText { text } => text,
            _ => unreachable!(
                "`source_idx` must point to either `LocatedTokenText` or `DynamicText` in the `elements`."
            ),
        }
    }

    /// Check if this import should be ignored (not sorted).
    pub fn is_ignored(&self, options: options::SortImports) -> bool {
        match self.import_line {
            SourceLine::Import(ImportLine { is_side_effect, .. }) => {
                // TODO: Check ignore comments?
                !options.sort_side_effects && is_side_effect
            }
            _ => unreachable!("`import_line` must be of type `SourceLine::Import`."),
        }
    }
}
