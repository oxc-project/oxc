mod compute_metadata;
mod group_config;
mod group_matcher;
pub mod options;
mod partitioned_chunk;
mod sortable_imports;
mod source_line;

use oxc_allocator::{Allocator, Vec as ArenaVec};

use crate::{
    Buffer, JsLabels, SortImportsOptions,
    formatter::{
        Formatter,
        format_element::{
            FormatElement, LineMode,
            tag::{LabelId, Tag},
        },
    },
    ir_transform::sort_imports::{
        group_matcher::GroupMatcher, partitioned_chunk::PartitionedChunk, source_line::SourceLine,
    },
};

/// Sort a chunk of `ImportDeclaration`s and replace the original `FormatElement`s with the sorted ones
/// in the formatter buffer.
///
/// `chunk_start` is the buffer position of the start of the chunk.
/// i.e. the buffer position recorded just before the first `entry` call for first `ImportDeclaration` in the chunk.
///
/// The chunk buffer must contain nothing but the `FormatElement`s generated from `ImportDeclaration`s,
/// and their preceding/trailing comments, and line breaks.
///
/// The chunk slice does not include any trailing line break, and `transform` produces a slice with
/// the same shape. The inter-statement separator is written by the next `entry` call
/// (or the closing newline emitted by `Program::write`).
///
/// The caller must already have verified that `sort_imports` option is enabled.
///
/// # Panics
/// Panics if `sort_imports` option is not enabled.
pub fn sort_imports_chunk(formatter: &mut Formatter<'_, '_>, chunk_start: usize) {
    let elements = &formatter.elements()[chunk_start..];
    let options = formatter.options().sort_imports.as_ref().unwrap();

    let sorted_elements = transform(elements, options, formatter.allocator());
    formatter.replace_end(chunk_start, &sorted_elements);
}

/// An IR transform that sorts import statements according to specified options.
/// Heavily inspired by ESLint's `@perfectionist/sort-imports` rule.
/// <https://perfectionist.dev/rules/sort-imports>
///
/// Transform the given slice of `FormatElement`s by sorting import statements according
/// to the specified options. The slice is one chunk of consecutive `ImportDeclaration`s
/// as written to the formatter buffer (delimited by the streaming driver in `FormatProgramBody`).
///
// NOTE: The input `FormatElement`s are already well-formatted.
// It means that:
// - There is no redundant spaces, no consecutive line breaks, etc...
// - The slice does not end with a line break (the inter-statement separator is written
//   by the next `entry` call, not as part of the chunk).
fn transform<'a>(
    elements: &[FormatElement<'a>],
    options: &SortImportsOptions,
    allocator: &'a Allocator,
) -> ArenaVec<'a, FormatElement<'a>> {
    // Parse string based groups into our internal representation for performance
    let group_matcher = GroupMatcher::new(&options.groups, &options.custom_groups);
    let prev_elements: &[FormatElement<'a>] = elements;

    // Roughly speaking, sort-imports is a process of swapping lines.
    // Therefore, as a preprocessing, group IR elements into line first.
    // e.g.
    // ```text
    // [Text, Space, Text, Line, StartTag, Text, Text, EndTag, Line, ...]
    // ```
    // ↓↓
    // ```text
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

    // All flushed lines terminate with `Hard`.
    // The `Line(Empty)` semantic is captured separately as a `SourceLine::Empty` push,
    // so we don't propagate `Empty` mode into the line itself.
    let build_flush_line = |range, standalone_alignable_comment| {
        if standalone_alignable_comment {
            SourceLine::CommentOnly(range, LineMode::Hard)
        } else {
            SourceLine::from_element_range(prev_elements, range, LineMode::Hard)
        }
    };

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
            // don't flush the line. e.g.
            // ```text
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
                lines.push(build_flush_line(
                    current_line_start..idx,
                    is_standalone_alignable_comment,
                ));
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
    // Flush the final line at end-of-input.
    // The chunk doesn't end with a `Line(Hard)`, so the in-loop flush above won't catch this.
    if current_line_start < prev_elements.len() {
        debug_assert!(
            !in_alignable_block_comment && !inside_multiline_import,
            "Unbalanced labelled tags at end of chunk"
        );
        lines.push(build_flush_line(
            current_line_start..prev_elements.len(),
            is_standalone_alignable_comment,
        ));
    }

    // Next, partition `SourceLine`s into `PartitionedChunk`s.
    //
    // Chunking is done by detecting boundaries.
    // By default, no boundary exists within an import run, so the whole chunk is sorted as one unit.
    // Depending on options, empty lines (`partition_by_newline`) and comment-only lines
    // (`partition_by_comment`) can also be boundaries.
    //
    // Within each chunk, we will sort import lines.
    // e.g. with `partition_by_newline: true`:
    // ```text
    // import C from "c"; // chunk1
    // import B from "b"; // chunk1
    //
    // import Z from "z"; // chunk2
    // import A from "a"; // chunk2
    // ```
    // ↓↓
    // ```text
    // import B from "b"; // chunk1
    // import C from "c"; // chunk1
    //
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
            // Reached only when `partition_by_newline` is true (for `Empty`)
            // or `partition_by_comment` is true (for `CommentOnly`).
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
                // ```text
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
                let (sorted_imports, orphan_contents, trailing_lines, slot_had_leading_blank) =
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
                let mut seen_non_ignored = false;
                for (slot_idx, sorted_import) in sorted_imports.iter().enumerate() {
                    // Insert newline when:
                    // 1. Group changes
                    // 2. At least one non-ignored import has already been output
                    //    (leading ignored imports don't trigger group boundaries)
                    // 3. The boundary override (or global `newlines_between`) says to insert
                    //    For decreasing transitions, fall back to whether the original input
                    //    had a blank line at this slot position (matches perfectionist).
                    let current_group_idx = sorted_import.group_idx;
                    if let Some(prev_idx) = prev_group_idx
                        && prev_idx != current_group_idx
                        && seen_non_ignored
                        && should_insert_newline_between(
                            options.newlines_between,
                            &options.newline_boundary_overrides,
                            prev_idx,
                            current_group_idx,
                            slot_had_leading_blank[slot_idx],
                        )
                    {
                        next_elements.push(FormatElement::Line(LineMode::Empty));
                    }
                    prev_group_idx = Some(current_group_idx);
                    if !sorted_import.is_ignored {
                        seen_non_ignored = true;
                    }

                    // Output leading lines and import line
                    for line in &sorted_import.leading_lines {
                        line.write(prev_elements, &mut next_elements, options.partition_by_newline);
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
                // We should preserve it only if the next chunk is a boundary
                // (i.e. a `partition_by_comment` comment-only line).
                // e.g. with `partition_by_comment: true`:
                // ```text
                // import A from "a"; // chunk1
                // import B from "b"; // chunk1
                // // This empty line should be preserved because the next chunk is a boundary.
                //
                // // BOUNDARY-COMMENT
                // import C from "c"; // chunk2
                // ```
                // But in this case, we should not preserve it.
                // ```text
                // import A from "a"; // chunk1
                // import B from "b"; // chunk1
                // // This empty line should NOT be preserved because the next chunk is NOT a boundary.
                //
                // import C from "c"; // chunk2
                // ```
                let next_chunk_is_boundary =
                    chunks_iter.peek().is_some_and(|c| matches!(c, PartitionedChunk::Boundary(_)));
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

    // The last `SourceLine::write` always pushes a trailing line break,
    // but the input chunk had none. Pop it to match the input shape.
    debug_assert!(matches!(next_elements.last(), Some(FormatElement::Line(_))));
    next_elements.pop();

    next_elements
}

/// Resolve whether a blank line should be inserted between two group indices.
/// Checks each boundary between `prev_group_idx` and `current_group_idx`,
/// using per-boundary overrides if available, otherwise the global `newlines_between`.
///
/// When groups are skipped (i.e. no imports match an intermediate group),
/// multiple boundaries are evaluated with OR semantics.
/// If any single boundary in the range resolves to `true`, a blank line is inserted.
///
/// Decreasing transitions (`prev_group_idx > current_group_idx`) can occur when
/// ignored (side-effect) imports preserve their original positions while other
/// imports are sorted. To match perfectionist's behavior, the configured boundary
/// rules are not enforced in that direction; instead, the original blank-line state
/// at this slot is preserved via `slot_had_leading_blank`.
fn should_insert_newline_between(
    global_newlines_between: bool,
    newline_boundary_overrides: &[Option<bool>],
    prev_group_idx: usize,
    current_group_idx: usize,
    slot_had_leading_blank: bool,
) -> bool {
    if prev_group_idx > current_group_idx {
        return slot_had_leading_blank;
    }

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
