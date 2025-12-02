mod compute_metadata;
mod group_config;
pub mod options;
mod partitioned_chunk;
mod sortable_imports;
mod source_line;

use oxc_allocator::{Allocator, Vec as ArenaVec};

use crate::{
    formatter::format_element::{FormatElement, LineMode, document::Document},
    ir_transform::sort_imports::{
        group_config::{GroupName, parse_groups_from_strings},
        partitioned_chunk::PartitionedChunk,
        source_line::SourceLine,
    },
};

/// An IR transform that sorts import statements according to specified options.
/// Heavily inspired by ESLint's `@perfectionist/sort-imports` rule.
/// <https://perfectionist.dev/rules/sort-imports>
pub struct SortImportsTransform {
    options: options::SortImportsOptions,
    groups: Vec<Vec<GroupName>>,
}

impl SortImportsTransform {
    pub fn new(options: options::SortImportsOptions) -> Self {
        // Parse string based groups into our internal representation for performance
        let groups = parse_groups_from_strings(&options.groups);
        Self { options, groups }
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
        //   - If this is the case, we should check `Tag::StartLabelled(JsLabels::ImportDeclaration)`
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
        // Chunking is done by detecting boundaries.
        // By default, only non-import lines are considered boundaries.
        // And depending on options, empty lines and comment-only lines can also be boundaries.
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
        while let Some((_idx, chunk)) = chunks_iter.next() {
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
                    let (sorted_imports, trailing_lines) =
                        chunk.into_sorted_import_units(&self.groups, &self.options);

                    // Output sorted import units
                    let mut prev_group_idx = None;
                    let mut prev_was_ignored = false;
                    for sorted_import in sorted_imports {
                        // Insert newline when:
                        // 1. Group changes
                        // 2. Previous import was not ignored (don't insert after ignored)
                        if self.options.newlines_between {
                            let current_group_idx = sorted_import.group_idx;
                            if let Some(prev_idx) = prev_group_idx
                                && prev_idx != current_group_idx
                                && !prev_was_ignored
                            {
                                next_elements.push(FormatElement::Line(LineMode::Empty));
                            }
                            prev_group_idx = Some(current_group_idx);
                            prev_was_ignored = sorted_import.is_ignored;
                        }

                        // Output leading lines and import line
                        for line in sorted_import.leading_lines {
                            line.write(
                                prev_elements,
                                &mut next_elements,
                                self.options.partition_by_newline,
                            );
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
                        line.write(
                            prev_elements,
                            &mut next_elements,
                            if is_last_empty_line { next_chunk_is_boundary } else { true },
                        );
                    }
                }
            }
        }

        Document::from(next_elements)
    }
}
