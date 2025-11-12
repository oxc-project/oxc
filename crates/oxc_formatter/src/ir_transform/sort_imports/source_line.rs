use std::ops::Range;

use crate::{
    JsLabels,
    formatter::format_element::{
        FormatElement, LineMode,
        tag::{LabelId, Tag},
    },
};

#[derive(Debug, Clone)]
pub enum SourceLine {
    /// Line that contains an import statement.
    /// May have leading comments like `/* ... */ import ...`.
    /// And also may have trailing comments like `import ...; // ...`.
    ///
    /// Never be a boundary.
    Import(Range<usize>, ImportLineMetadata),
    /// Empty line.
    /// May be used as a boundary if `options.partition_by_newline` is true.
    Empty,
    /// Line that contains only comment(s).
    /// May be used as a boundary if `options.partition_by_comment` is true.
    CommentOnly(Range<usize>, LineMode),
    /// Other lines, always a boundary.
    Others(Range<usize>, LineMode),
}

impl SourceLine {
    pub fn from_element_range(
        elements: &[FormatElement],
        range: Range<usize>,
        line_mode: LineMode,
    ) -> Self {
        debug_assert!(
            !range.is_empty(),
            "`range` must not be empty, otherwise use `SourceLine::Empty` directly."
        );

        // Check if the line is comment-only.
        // e.g.
        // ```
        // // comment
        // /* comment */
        // /* comment */ // comment
        // /* comment */ /* comment */
        // ```
        let is_comment_only = range.clone().all(|idx| match &elements[idx] {
            FormatElement::Text { text, width } => text.starts_with("//") || text.starts_with("/*"),
            FormatElement::Line(LineMode::Soft | LineMode::SoftOrSpace) | FormatElement::Space => {
                true
            }
            _ => false,
        });
        if is_comment_only {
            // TODO: Check it contains ignore comment?
            return SourceLine::CommentOnly(range, line_mode);
        }

        // Check if the line contains an import statement.
        // Sometimes, there might be leading comments in the same line,
        // so we need to check all elements in the line to find an `ImportDeclaration`.
        // ```
        // /* THIS */ import ...
        // import ...
        // ```
        let mut has_import = false;
        let mut source_idx = None;
        let mut is_side_effect = true;
        let mut is_type_import = false;
        let mut has_default_specifier = false;
        let mut has_namespace_specifier = false;
        let mut has_named_specifier = false;

        for idx in range.clone() {
            let element = &elements[idx];

            // Special marker for `ImportDeclaration`
            if let FormatElement::Tag(Tag::StartLabelled(id)) = element {
                if *id == LabelId::of(JsLabels::ImportDeclaration) {
                    has_import = true;
                }
                continue;
            }
            if !has_import {
                continue;
            }

            match element {
                FormatElement::Token { text } => match *text {
                    "import" => {
                        // Look ahead to determine import type (skip spaces)
                        let mut offset = 1;
                        while idx + offset < elements.len() {
                            if matches!(elements[idx + offset], FormatElement::Space) {
                                offset += 1;
                                continue;
                            }

                            match &elements[idx + offset] {
                                FormatElement::Token { text } => match *text {
                                    "type" => is_type_import = true,
                                    "*" => has_namespace_specifier = true,
                                    "{" => has_named_specifier = true,
                                    _ => {}
                                },
                                FormatElement::Text { .. } => {
                                    has_default_specifier = true;
                                }
                                _ => {}
                            }
                            break;
                        }
                    }
                    "from" => {
                        is_side_effect = false;
                        source_idx = None;
                    }
                    _ => {}
                },
                FormatElement::Text { .. } => {
                    if source_idx.is_none() {
                        source_idx = Some(idx);
                    }
                }
                _ => {}
            }
        }

        if has_import && let Some(source_idx) = source_idx {
            // TODO: Check line has trailing ignore comment?
            return SourceLine::Import(
                range,
                ImportLineMetadata {
                    source_idx,
                    is_side_effect,
                    is_type_import,
                    has_default_specifier,
                    has_namespace_specifier,
                    has_named_specifier,
                },
            );
        }

        // Otherwise, this line is neither of:
        // - Empty line
        // - Comment-only line
        // - Import line
        // So, it will be a boundary line.
        SourceLine::Others(range, line_mode)
    }

    /// Collect element indices that this line should emit, along with the line mode.
    /// Returns `(element_indices, line_mode_to_append)` where:
    /// - `element_indices` are indices into the original elements array
    /// - `line_mode_to_append` is the line break to add after these elements (None means skip)
    pub fn element_indices(&self, preserve_empty_line: bool) -> (Range<usize>, Option<LineMode>) {
        match self {
            SourceLine::Empty => {
                // Empty range, and optionally emit Empty line mode
                let mode = if preserve_empty_line { Some(LineMode::Empty) } else { None };
                (0..0, mode)
            }
            SourceLine::Import(range, _) => {
                // Always use hard line break after import statement
                (range.clone(), Some(LineMode::Hard))
            }
            SourceLine::CommentOnly(range, mode) | SourceLine::Others(range, mode) => {
                (range.clone(), Some(*mode))
            }
        }
    }
}

/// Import line metadata extracted during parsing.
#[derive(Debug, Clone)]
pub struct ImportLineMetadata {
    /// Index of the import source in the original `elements` slice.
    pub source_idx: usize,
    /// Whether this is a side-effect-only import (e.g., `import "foo"`).
    pub is_side_effect: bool,
    /// Whether this is a type-only import (e.g., `import type { Foo } from "foo"`).
    pub is_type_import: bool,
    /// Whether this import has a default specifier (e.g., `import Foo from "foo"`).
    pub has_default_specifier: bool,
    /// Whether this import has a namespace specifier (e.g., `import * as Foo from "foo"`).
    pub has_namespace_specifier: bool,
    /// Whether this import has named specifiers (e.g., `import { foo } from "foo"`).
    pub has_named_specifier: bool,
}
