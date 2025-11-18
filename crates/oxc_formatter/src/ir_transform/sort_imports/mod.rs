mod import_unit;
mod partitioned_chunk;
mod source_line;

use std::mem;

use oxc_allocator::{Allocator, Vec as ArenaVec};

use crate::{
    formatter::format_element::{FormatElement, LineMode, document::Document},
    options,
};

use import_unit::SortableImport;
use partitioned_chunk::PartitionedChunk;
use source_line::SourceLine;

pub struct SortImportsTransform {
    options: options::SortImports,
}

impl SortImportsTransform {
    pub fn new(options: options::SortImports) -> Self {
        Self { options }
    }

    /// Transform the given `Document` by sorting import statements according to the specified options.
    ///
    // NOTE: `Document` and its `FormatElement`s are already well-formatted.
    // It means that:
    // - There is no redundant spaces, no consecutive line breaks, etc...
    // - Last element is always `FormatElement::Line(Hard)`.
    pub fn transform<'a>(&self, document: &Document<'a>, allocator: &'a Allocator) -> Document<'a> {
        // Early return for empty files
        if document.len() == 1 && matches!(document[0], FormatElement::Line(LineMode::Hard)) {
            return document.clone();
        }

        let prev_elements: &[FormatElement<'a>] = document;

        // Roughly speaking, sort-imports is a process of swapping lines.
        // Therefore, as a preprocessing, group IR elements into line first.
        // e.g.
        // ```
        // [Text, Space, Text, Line, StartTag, Text, Text, EndTag, Line, ...]
        // ```
        // ↓↓
        // ```
        // [ [Text, Space, Text], [StartTag, Text, Text, EndTag], [...] ]
        // ```
        //
        // This is also meaningful to explicitly handle comment line, empty line,
        // and other line with or without import statement.
        //
        // NOTE: `FormatElement::Line(_)` may not exactly correspond to an actual line break in the output.
        // e.g. `LineMode::SoftOrSpace` may be rendered as a space.
        //
        // And this implementation is based on the following assumptions:
        // - Only `Line(Hard|Empty)` is used for joining `Program.body` in the output
        // - `Line(Hard|Empty)` does not appear inside an `ImportDeclaration` formatting
        //   - In case of this, we should check `Tag::StartLabelled(JsLabels::ImportDeclaration)`
        let mut lines = vec![];
        let mut current_line_start = 0;
        for (idx, el) in prev_elements.iter().enumerate() {
            if let FormatElement::Line(mode) = el
                && matches!(mode, LineMode::Empty | LineMode::Hard)
            {
                // Flush current line
                if current_line_start < idx {
                    lines.push(SourceLine::from_element_range(
                        prev_elements,
                        current_line_start..idx,
                        *mode,
                    ));
                }
                current_line_start = idx + 1;

                // We need this explicitly to detect boundaries later.
                if matches!(mode, LineMode::Empty) {
                    lines.push(SourceLine::Empty);
                }
            }
        }
        if current_line_start < prev_elements.len() {
            unreachable!("`Document` must end with a `FormatElement::Line(Hard)`.");
        }

        // Next, partition `SourceLine`s into `PartitionedChunk`s.
        //
        // Within each chunk, we will sort import lines.
        // e.g.
        // ```
        // import C from "c"; // chunk1
        // import B from "b"; // chunk1
        // const THIS_IS_BOUNDARY = true;
        // import Z from "z"; // chunk2
        // import A from "a"; // chunk2
        // ```
        // ↓↓
        // ```
        // import B from "b"; // chunk1
        // import C from "c"; // chunk1
        // const THIS_IS_BOUNDARY = true;
        // import A from "a"; // chunk2
        // import Z from "z"; // chunk2
        // ```
        let mut chunks = vec![];
        let mut current_chunk = PartitionedChunk::default();
        for line in lines {
            match line {
                // `SourceLine::Import` never be a boundary.
                SourceLine::Import(..) => {
                    current_chunk.add_imports_line(line);
                }
                // `SourceLine::Empty` and `SourceLine::CommentOnly` can be boundaries depending on options.
                // Otherwise, they will be the leading/trailing lines of `PartitionedChunk::Imports`.
                SourceLine::Empty if !self.options.partition_by_newline => {
                    current_chunk.add_imports_line(line);
                }
                // TODO: Support more flexible comment handling?
                // e.g. Specific text by regex, only line comments, etc.
                SourceLine::CommentOnly(..) if !self.options.partition_by_comment => {
                    current_chunk.add_imports_line(line);
                }
                // This `SourceLine` is a boundary!
                // Generally, `SourceLine::Others` should always reach here.
                _ => {
                    // Flush current import chunk
                    if !current_chunk.is_empty() {
                        chunks.push(std::mem::take(&mut current_chunk));
                    }
                    // Add boundary chunk
                    chunks.push(PartitionedChunk::Boundary(line));
                }
            }
        }
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        // Finally, sort import lines within each chunk.
        // After sorting, flatten everything back to `FormatElement`s.
        let mut next_elements = ArenaVec::with_capacity_in(prev_elements.len(), allocator);

        let mut chunks_iter = chunks.into_iter().enumerate().peekable();
        while let Some((idx, chunk)) = chunks_iter.next() {
            match chunk {
                // Boundary chunks: Just output as-is
                PartitionedChunk::Boundary(line) => {
                    line.write(prev_elements, &mut next_elements, true);
                }
                // Import chunks: Sort and output
                PartitionedChunk::Imports(_) => {
                    // For ease of implementation, we will convert `ImportChunk` into multiple `SortableImport`s.
                    //
                    // `SortableImport` is a logical unit of 1 import statement + its N leading lines.
                    // And there may be trailing lines after all import statements in the chunk.
                    // e.g.
                    // ```
                    // const THIS_IS_BOUNDARY = true;
                    // // comment for A
                    // import A from "a"; // sortable1
                    // import B from "b"; // sortable2
                    //
                    // // comment for C and empty line above + below
                    //
                    // // another comment for C
                    // import C from "c"; // sortable3
                    // // trailing comment and empty line below for this chunk
                    //
                    // const YET_ANOTHER_BOUNDARY = true;
                    // ```
                    let (mut sortable_imports, trailing_lines) =
                        chunk.into_import_units(prev_elements, &self.options);

                    sort_imports(&mut sortable_imports, &self.options);

                    // Output sorted import units
                    let preserve_empty_line = self.options.partition_by_newline;
                    let mut prev_group_idx = None;
                    for sorted_import in sortable_imports {
                        // Insert blank line between different groups if enabled
                        if self.options.newlines_between {
                            let current_group_idx = sorted_import.group_idx;
                            if let Some(prev_idx) = prev_group_idx
                                && prev_idx != current_group_idx
                            {
                                next_elements.push(FormatElement::Line(LineMode::Empty));
                            }
                            prev_group_idx = Some(current_group_idx);
                        }

                        // Output leading lines and import line
                        for line in sorted_import.leading_lines {
                            line.write(prev_elements, &mut next_elements, preserve_empty_line);
                        }
                        sorted_import.import_line.write(prev_elements, &mut next_elements, false);
                    }
                    // And output trailing lines
                    //
                    // Special care is needed for the last empty line.
                    // We should preserve it only if the next chunk is a boundary.
                    // e.g.
                    // ```
                    // import A from "a"; // chunk1
                    // import B from "b"; // chunk1
                    // // This empty line should be preserved because the next chunk is a boundary.
                    //
                    // const BOUNDARY = true; // chunk2
                    // ```
                    // But in this case, we should not preserve it.
                    // ```
                    // import A from "a"; // chunk1
                    // import B from "b"; // chunk1
                    // // This empty line should NOT be preserved because the next chunk is NOT a boundary.
                    //
                    // import C from "c"; // chunk2
                    // ```
                    let next_chunk_is_boundary = chunks_iter
                        .peek()
                        .is_some_and(|(_, c)| matches!(c, PartitionedChunk::Boundary(_)));
                    for (idx, line) in trailing_lines.iter().enumerate() {
                        let is_last_empty_line =
                            idx == trailing_lines.len() - 1 && matches!(line, SourceLine::Empty);
                        let preserve_empty_line =
                            if is_last_empty_line { next_chunk_is_boundary } else { true };
                        line.write(prev_elements, &mut next_elements, preserve_empty_line);
                    }
                }
            }
        }

        Document::from(next_elements)
    }
}

/// Sort a list of imports in-place according to the given options.
fn sort_imports(imports: &mut [SortableImport], options: &options::SortImports) {
    let imports_len = imports.len();

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
    for (idx, si) in imports.iter().enumerate() {
        if si.is_ignored {
            fixed_indices.push(idx);
        } else {
            sortable_indices.push(idx);
        }
    }

    // Sort indices by comparing their corresponding import groups, then sources.
    sortable_indices.sort_by(|&a, &b| {
        // Always sort by groups array order first
        let group_ord = imports[a].group_idx.cmp(&imports[b].group_idx);
        if group_ord != std::cmp::Ordering::Equal {
            return group_ord;
        }

        // Within the same group, sort by source respecting the order option
        let source_ord = imports[a].normalized_source.cmp(&imports[b].normalized_source);
        if options.order.is_desc() { source_ord.reverse() } else { source_ord }
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
            imports.swap(current, next);
            current = next;
        }
    }
    debug_assert!(imports.len() == imports_len, "Length must remain the same after sorting.");
}
