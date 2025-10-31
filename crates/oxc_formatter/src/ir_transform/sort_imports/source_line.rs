use std::ops::Range;

use crate::{
    JsLabels,
    formatter::format_element::{
        FormatElement, LineMode,
        tag::{LabelId, Tag},
    },
};

/// Import line information.
#[derive(Debug, Clone)]
pub struct ImportLine {
    /// Range of elements in the original `elements` slice.
    pub elements_range: Range<usize>,
    /// Index of the import source in the original `elements` slice.
    pub source_idx: usize,
    /// Whether this is a side-effect-only import (e.g., `import "foo"`).
    pub is_side_effect: bool,
}

#[derive(Debug, Clone)]
pub enum SourceLine {
    /// Line that contains an import statement.
    /// May have leading comments like `/* ... */ import ...`.
    /// And also may have trailing comments like `import ...; // ...`.
    ///
    /// Never be a boundary.
    Import(ImportLine),
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
            FormatElement::DynamicText { text } => text.starts_with("//") || text.starts_with("/*"),
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
        for idx in range.clone() {
            match &elements[idx] {
                // Special marker for `ImportDeclaration`
                FormatElement::Tag(Tag::StartLabelled(id))
                    if *id == LabelId::of(JsLabels::ImportDeclaration) =>
                {
                    has_import = true;
                }
                FormatElement::StaticText { text } => {
                    if has_import && *text == "from" {
                        is_side_effect = false;
                        // Reset `source_idx` to ensure we get the text after "from".
                        // `ImportSpecifier` may appear before `source`.
                        source_idx = None;
                    }
                }
                // `ImportDeclaration.source: StringLiteral` is formatted as either:
                // - `LocatedTokenText` (when borrowed, quote unchanged)
                // - `DynamicText` (when owned, quote normalized)
                FormatElement::LocatedTokenText { .. } | FormatElement::DynamicText { .. } => {
                    if has_import && source_idx.is_none() {
                        source_idx = Some(idx);
                    }
                }
                _ => {}
            }
        }
        if has_import && let Some(source_idx) = source_idx {
            // TODO: Check line has trailing ignore comment?
            return SourceLine::Import(ImportLine {
                elements_range: range,
                source_idx,
                is_side_effect,
            });
        }

        // Otherwise, this line is neither of:
        // - Empty line
        // - Comment-only line
        // - Import line
        // So, it will be a boundary line.
        SourceLine::Others(range, line_mode)
    }

    pub fn write<'a>(
        &self,
        prev_elements: &[FormatElement<'a>],
        next_elements: &mut Vec<FormatElement<'a>>,
        preserve_empty_line: bool,
    ) {
        match self {
            SourceLine::Empty => {
                // Skip empty lines unless they should be preserved
                if preserve_empty_line {
                    next_elements.push(FormatElement::Line(LineMode::Empty));
                }
            }
            SourceLine::Import(ImportLine { elements_range, .. }) => {
                for idx in elements_range.clone() {
                    next_elements.push(prev_elements[idx].clone());
                }
                // Always use hard line break after import statement.
                next_elements.push(FormatElement::Line(LineMode::Hard));
            }
            SourceLine::CommentOnly(range, mode) | SourceLine::Others(range, mode) => {
                for idx in range.clone() {
                    next_elements.push(prev_elements[idx].clone());
                }
                next_elements.push(FormatElement::Line(*mode));
            }
        }
    }
}
