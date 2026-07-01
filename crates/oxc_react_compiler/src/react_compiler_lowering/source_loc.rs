//! Source-location index for the oxc-AST front-end.
//!
//! Replaces the `loc` / `node_id` synthesis that `convert_ast` performed while
//! building the Babel-shaped AST. oxc nodes carry byte-offset [`Span`]s; the HIR
//! wants [`SourceLocation`] with `line` / `column` / `index`. This computes
//! line/column from a one-time line-offset table, byte-for-byte identical to the
//! table `convert_ast::ConvertCtx` built, so HIR locations are unchanged.
//!
//! Invariant carried over from the Babel bridge: `node_id == span.start`, and
//! `column` is a **byte** offset from the line start (not UTF-16), matching what
//! the previous pipeline produced.

use oxc_span::Span;

use crate::react_compiler_hir::Position;
use crate::react_compiler_hir::SourceLocation;

/// One-time index of line-start byte offsets, for `Span` → [`SourceLocation`].
#[allow(dead_code)]
pub struct LineOffsets {
    /// Byte offset of the start of each line. `line_offsets[0] == 0`.
    line_offsets: Vec<u32>,
}

#[allow(dead_code)]
impl LineOffsets {
    pub fn new(source_text: &str) -> Self {
        let mut line_offsets = vec![0];
        for (i, ch) in source_text.char_indices() {
            if ch == '\n' {
                line_offsets.push((i + 1) as u32);
            }
        }
        Self { line_offsets }
    }

    /// Byte offset → 1-based line, byte-based column, byte index.
    pub fn position(&self, offset: u32) -> Position {
        let line_idx = match self.line_offsets.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.line_offsets[line_idx];
        Position { line: (line_idx + 1) as u32, column: offset - line_start, index: Some(offset) }
    }

    /// A [`Span`]'s byte range → an HIR [`SourceLocation`].
    pub fn source_location(&self, span: Span) -> SourceLocation {
        SourceLocation { start: self.position(span.start), end: self.position(span.end) }
    }
}
