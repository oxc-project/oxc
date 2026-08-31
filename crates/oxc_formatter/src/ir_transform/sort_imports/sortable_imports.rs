use std::borrow::Cow;

use crate::ir_transform::{
    sort_common::permutation::{apply_permutation, group_sort, sort_with_pinned},
    sort_imports::{options::SortImportsOptions, source_line::SourceLine},
};

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
        if imports_len < 2 {
            return;
        }

        // NOTE: Apply `desc` by reversing the per-comparison `Ordering`,
        // NOT by reversing the sorted slice afterwards.
        // `reverse()` on the slice would also flip imports that compare `Equal` (e.g. same source),
        // breaking stability and making sorting non-idempotent.
        let compare_sources = |a: usize, b: usize| {
            let ordering = natord::compare(&self[a].normalized_source, &self[b].normalized_source);
            if options.order.is_desc() { ordering.reverse() } else { ordering }
        };

        // Ignored imports (side-effects that may not be regrouped) keep their absolute position.
        // Inside a group, side-effect imports keep their relative position unless `sort_side_effects`.
        let permutation = group_sort(
            imports_len,
            |idx| self[idx].group_idx,
            |idx| self[idx].is_ignored,
            |group_indices| {
                if options.sort_side_effects {
                    group_indices.sort_by(|&a, &b| compare_sources(a, b));
                } else {
                    sort_with_pinned(
                        group_indices,
                        |idx| self[idx].is_side_effect,
                        compare_sources,
                    );
                }
            },
        );

        apply_permutation(self, &permutation);
    }
}
