use std::ops::Range;

use oxc_allocator::Vec as ArenaVec;

use crate::formatter::format_element::{FormatElement, ImportDeclMetadata, LineMode};

#[derive(Debug)]
pub enum SourceLine<'a> {
    /// Line that contains an import statement.
    /// May have leading comments like `/* ... */ import ...`.
    /// And also may have trailing comments like `import ...; // ...`.
    /// Never be a boundary.
    Import(Range<usize>, &'a ImportDeclMetadata<'a>),
    /// Empty line.
    /// May be used as a boundary if `options.partition_by_newline` is true.
    Empty,
    /// Line that contains only comment(s).
    /// May be used as a boundary if `options.partition_by_comment` is true.
    CommentOnly(Range<usize>, LineMode),
    /// Other lines, always a boundary.
    Others(Range<usize>, LineMode),
}

impl<'a> SourceLine<'a> {
    pub fn from_element_range(
        elements: &[FormatElement<'a>],
        range: Range<usize>,
        line_mode: LineMode,
    ) -> Self {
        debug_assert!(
            !range.is_empty(),
            "`range` must not be empty, otherwise use `SourceLine::Empty` directly."
        );

        // Check if the line contains an import by looking for ImportMetadata element.
        // This metadata was attached during formatting from AST information,
        // so no token re-parsing is needed.
        for idx in range.clone() {
            if let FormatElement::ImportMetadata(metadata) = &elements[idx] {
                // TODO: Check line has trailing ignore comment?
                return SourceLine::Import(range, metadata);
            }
        }

        // Check if the line is comment-only.
        // e.g.
        // ```text
        // // comment
        // /* comment */
        // /* comment */ // comment
        // /* comment */ /* comment */
        // ```
        let is_comment_only = range.clone().all(|idx| match &elements[idx] {
            FormatElement::Text { text, width: _ } => {
                text.starts_with("//") || text.starts_with("/*")
            }
            FormatElement::Line(LineMode::Soft | LineMode::SoftOrSpace) | FormatElement::Space => {
                true
            }
            _ => false,
        });
        if is_comment_only {
            return SourceLine::CommentOnly(range, line_mode);
        }

        // Otherwise, this line is neither of:
        // - Empty line
        // - Comment-only line
        // - Import line
        // So, it will be a boundary line.
        SourceLine::Others(range, line_mode)
    }

    pub fn write(
        &self,
        prev_elements: &[FormatElement<'a>],
        next_elements: &mut ArenaVec<'a, FormatElement<'a>>,
        preserve_empty_line: bool,
    ) {
        match self {
            SourceLine::Empty => {
                // Skip empty lines unless they should be preserved
                if preserve_empty_line {
                    next_elements.push(FormatElement::Line(LineMode::Empty));
                }
            }
            SourceLine::Import(range, _) => {
                for idx in range.clone() {
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
