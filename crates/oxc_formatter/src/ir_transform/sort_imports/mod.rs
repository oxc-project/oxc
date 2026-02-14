mod compute_metadata;
mod group_config;
mod group_matcher;
pub mod options;
mod partitioned_chunk;
mod sortable_imports;
mod source_line;

use oxc_allocator::{Allocator, Vec as ArenaVec};

use crate::{
    JsLabels, SortImportsOptions,
    formatter::format_element::{
        FormatElement, LineMode,
        document::Document,
        tag::{LabelId, Tag},
    },
    ir_transform::sort_imports::{
        group_matcher::GroupMatcher, partitioned_chunk::PartitionedChunk, source_line::SourceLine,
    },
};

/// An IR transform that sorts import statements according to specified options.
/// Heavily inspired by ESLint's `@perfectionist/sort-imports` rule.
/// <https://perfectionist.dev/rules/sort-imports>
pub struct SortImportsTransform;

impl SortImportsTransform {
    /// Transform the given `Document` by sorting import statements according to the specified options.
    ///
    // NOTE: `Document` and its `FormatElement`s are already well-formatted.
    // It means that:
    // - There is no redundant spaces, no consecutive line breaks, etc...
    // - Last element is always `FormatElement::Line(Hard)`.
    pub fn transform<'a>(
        document: &Document<'a>,
        options: &SortImportsOptions,
        allocator: &'a Allocator,
    ) -> Option<ArenaVec<'a, FormatElement<'a>>> {
        // Early return for empty files
        if document.len() == 1 && matches!(document[0], FormatElement::Line(LineMode::Hard)) {
            return None;
        }

        // Parse string based groups into our internal representation for performance
        let group_matcher = GroupMatcher::new(&options.groups, &options.custom_groups);
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
        // Track if we're inside an alignable block comment (identified by `JsLabels::AlignableBlockComment`)
        let mut in_alignable_block_comment = false;
        // Track if current line is a standalone alignable comment (no import on same line)
        let mut is_standalone_alignable_comment = false;
        // Track if current line is inside a multiline ImportDeclaration
        let mut inside_multiline_import = false;

        for (idx, el) in prev_elements.iter().enumerate() {
            // Check for alignable block comment boundaries.
            // These comments are split across multiple lines with hard_line_break() between them,
            // so we need to track when we're inside one to avoid flushing lines prematurely.
            if let FormatElement::Tag(Tag::StartLabelled(id)) = el {
                if *id == LabelId::of(JsLabels::AlignableBlockComment) {
                    in_alignable_block_comment = true;
                    is_standalone_alignable_comment = true;
                } else if *id == LabelId::of(JsLabels::ImportDeclaration) {
                    inside_multiline_import = true;
                    // An import on the same line means the comment is attached to it, not standalone
                    is_standalone_alignable_comment = false;
                }
            } else if matches!(el, FormatElement::Tag(Tag::EndLabelled)) {
                // EndLabelled doesn't carry the label ID, but since AlignableBlockComment
                // doesn't nest with other labels in practice, we can safely reset here.
                if in_alignable_block_comment {
                    in_alignable_block_comment = false;
                } else if inside_multiline_import {
                    // I'm not sure if ImportDeclaration will nest with other labels,
                    // but this should be enough for now.
                    inside_multiline_import = false;
                }
            }

            if let FormatElement::Line(mode) = el
                && matches!(mode, LineMode::Empty | LineMode::Hard)
            {
                // If we're inside an alignable block comment, don't flush the line yet.
                // Wait until the comment is closed so the entire comment is treated as one line.
                if in_alignable_block_comment {
                    continue;
                }

                // If the linebreak falls within the body of a multiline ImportDeclaration,
                // don't fush the line. e.g.
                // ```
                // import React {
                //   useState,
                //   // this is a comment followed by a FormatElement::Line(LineMode::Hard)
                //   useEffect,
                // } from 'react';
                // ```
                if inside_multiline_import {
                    continue;
                }

                // Flush current line
                if current_line_start < idx {
                    let line = if is_standalone_alignable_comment {
                        // Standalone alignable comment: directly create CommentOnly
                        SourceLine::CommentOnly(current_line_start..idx, *mode)
                    } else {
                        SourceLine::from_element_range(
                            prev_elements,
                            current_line_start..idx,
                            *mode,
                        )
                    };
                    lines.push(line);
                }
                current_line_start = idx + 1;
                is_standalone_alignable_comment = false;
                // Explicitly reset the state after flushing lines to avoid stale state.
                inside_multiline_import = false;

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
                SourceLine::Empty if !options.partition_by_newline => {
                    current_chunk.add_imports_line(line);
                }
                // TODO: Support more flexible comment handling?
                // e.g. Specific text by regex, only line comments, etc.
                SourceLine::CommentOnly(..) if !options.partition_by_comment => {
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

        let mut chunks_iter = chunks.into_iter().peekable();
        while let Some(chunk) = chunks_iter.next() {
            match chunk {
                // Boundary chunks: Just output as-is
                PartitionedChunk::Boundary(line) => {
                    line.write(prev_elements, &mut next_elements, true);
                }
                // Import chunks: Sort and output
                PartitionedChunk::Imports(_) => {
                    // Convert `ImportChunk` into `SortableImport`s.
                    //
                    // `SortableImport` is a logical unit of 1 import statement with its leading lines.
                    // `OrphanContent` tracks comments separated by empty lines with their slot positions.
                    //
                    // Comments attach based on empty line separation:
                    // - Comments directly before an `import` (no empty line) → `SortableImport.leading_lines`
                    // - Comments followed by an empty line → `orphan_contents` (stay at slot position)
                    //
                    // e.g.
                    // ```
                    // // orphan (after_slot: None)
                    //
                    // // leading for A
                    // import A from "a";
                    // // orphan (after_slot: Some(0))
                    //
                    // // leading for B
                    // import B from "b";
                    // // chunk trailing
                    // ```
                    let (sorted_imports, orphan_contents, trailing_lines) =
                        chunk.into_sorted_import_units(&group_matcher, options);

                    // Output leading orphan content (after_slot: None)
                    for orphan in &orphan_contents {
                        if orphan.after_slot.is_none() {
                            for line in &orphan.lines {
                                line.write(prev_elements, &mut next_elements, true);
                            }
                        }
                    }

                    // Output sorted import units with orphan content at their slot positions
                    let mut prev_group_idx = None;
                    let mut prev_was_ignored = false;
                    for (slot_idx, sorted_import) in sorted_imports.iter().enumerate() {
                        // Insert newline when:
                        // 1. Group changes
                        // 2. Previous import was not ignored (don't insert after ignored)
                        // 3. The boundary override (or global `newlines_between`) says to insert
                        let current_group_idx = sorted_import.group_idx;
                        if let Some(prev_idx) = prev_group_idx
                            && prev_idx != current_group_idx
                            && !prev_was_ignored
                            && should_insert_newline_between(
                                options.newlines_between,
                                &options.newline_boundary_overrides,
                                prev_idx,
                                current_group_idx,
                            )
                        {
                            next_elements.push(FormatElement::Line(LineMode::Empty));
                        }
                        prev_group_idx = Some(current_group_idx);
                        prev_was_ignored = sorted_import.is_ignored;

                        // Output leading lines and import line
                        for line in &sorted_import.leading_lines {
                            line.write(
                                prev_elements,
                                &mut next_elements,
                                options.partition_by_newline,
                            );
                        }
                        sorted_import.import_line.write(prev_elements, &mut next_elements, false);

                        // Output orphan content that belongs after this slot
                        for orphan in &orphan_contents {
                            if orphan.after_slot == Some(slot_idx) {
                                for line in &orphan.lines {
                                    line.write(prev_elements, &mut next_elements, false);
                                }
                            }
                        }
                    }
                    // And output chunk's trailing lines
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
                        .is_some_and(|c| matches!(c, PartitionedChunk::Boundary(_)));
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

        Some(next_elements)
    }
}

/// Resolve whether a blank line should be inserted between two group indices.
/// Checks each boundary between `prev_group_idx` and `current_group_idx`,
/// using per-boundary overrides if available, otherwise the global `newlines_between`.
///
/// When groups are skipped (i.e. no imports match an intermediate group),
/// multiple boundaries are evaluated with OR semantics.
/// If any single boundary in the range resolves to `true`, a blank line is inserted.
fn should_insert_newline_between(
    global_newlines_between: bool,
    newline_boundary_overrides: &[Option<bool>],
    prev_group_idx: usize,
    current_group_idx: usize,
) -> bool {
    if newline_boundary_overrides.is_empty() {
        return global_newlines_between;
    }

    for idx in prev_group_idx..current_group_idx {
        if newline_boundary_overrides.get(idx).copied().flatten().unwrap_or(global_newlines_between)
        {
            return true;
        }
    }

    false
}
