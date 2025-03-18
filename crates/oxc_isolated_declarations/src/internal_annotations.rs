use std::cell::Cell;

use oxc_ast::ast::Program;
use oxc_span::Span;

/// Lookup table of the span starts of AST nodes with attached `@internal` comments.
///
/// `program.comments` is in ascending order of source position, and `span_starts` here is too.
/// If we work through the AST from top to bottom, we'll discover nodes with annotation comments
/// in ascending order of span start.
///
/// Therefore, we can avoid the overhead of a `HashMap` for lookups. We can store span starts in a `Vec`,
/// and keep a cursor that points to what the span start of the next comment is.
///
/// If we go back up to top of AST and loop through it again, we need to reset the cursor back
/// to the start before we do.
///
/// Implementation detail:
/// We use `Cell`s for `next_start` and `next_index` just to work around problems with borrow checker.
pub struct InternalAnnotations {
    /// Span starts of AST nodes with attached `@internal` comments.
    span_starts: Vec<u32>,
    /// Span start of next `@internal` comment.
    /// Equal to `self.span_starts[self.next_index]`, unless we've exhausted all comments,
    /// in which case it's `u32::MAX`.
    /// `u32::MAX` can never be the span start of an AST node, as source text is capped at `u32::MAX`
    /// bytes length, and an AST node cannot be zero width.
    next_start: Cell<u32>,
    /// Index in `span_starts` of next `@internal` comment.
    next_index: Cell<usize>,
    /// Last queried span start.
    /// Only for debug assertions which check that `has_annotation` is always called with spans
    /// in ascending order.
    #[cfg(debug_assertions)]
    last_query: Cell<Option<u32>>,
}

impl InternalAnnotations {
    /// Create empty [`InternalAnnotations`].
    pub fn new() -> Self {
        Self {
            span_starts: vec![],
            next_start: Cell::new(u32::MAX),
            next_index: Cell::new(0),
            #[cfg(debug_assertions)]
            last_query: Cell::new(None),
        }
    }

    /// Populate lookup table with span starts of nodes with attached `@internal` comments.
    ///
    /// This method leaves the cursor positioned on 1st comment.
    pub fn build(&mut self, program: &Program) {
        // `u32::MAX` cannot be the start of any node, because source text is limited to `u32::MAX` bytes
        let mut last_span_start = u32::MAX;
        for comment in &program.comments {
            let has_internal =
                comment.content_span().source_text(program.source_text).contains("@internal");
            debug_assert!(comment.attached_to >= last_span_start || last_span_start == u32::MAX);
            if has_internal && last_span_start != comment.attached_to {
                self.span_starts.push(comment.attached_to);
                last_span_start = comment.attached_to;
            }
        }

        if !self.span_starts.is_empty() {
            self.next_start = Cell::new(self.span_starts[0]);
        }
    }

    /// Reset cursor back to 1st comment.
    pub fn reset_cursor(&mut self) {
        // Note: We don't need to do anything if `span_starts` is empty, as state will be as it was
        // when `InternalAnnotations` was created - `next_index == 0` and `next_start == u32::MAX`
        if !self.span_starts.is_empty() {
            self.next_index = Cell::new(0);
            self.next_start = Cell::new(self.span_starts[0]);
        }

        #[cfg(debug_assertions)]
        self.last_query.set(None);
    }

    /// Check if the node with provided `Span` has an `@internal` annotation.
    ///
    /// Returns `true` if it does, and advances the cursor on to the next comment,
    /// ready for next lookup.
    #[inline]
    pub fn has_annotation(&self, span: Span) -> bool {
        // Check `has_annotation` is always called with spans in ascending order
        #[cfg(debug_assertions)]
        {
            let last_query = self.last_query.get();
            assert!(last_query.is_none() || span.start > last_query.unwrap());
            self.last_query.set(Some(span.start));
        }

        // This will always return `false` if `self.strip_internal` is `false` because then
        // `self.next_internal_annotation_start` is `u32::MAX`, which can never match `span.start`.
        // No need for an extra branch on `self.strip_internal`.
        let next_start = self.next_start.get();
        if span.start < next_start {
            return false;
        }

        self.find_next(span)
    }

    /// Advance the cursor until find an annotation which is after this one.
    ///
    /// Returns `true` if there is an annotation with matching span.
    fn find_next(&self, span: Span) -> bool {
        let mut next_index = self.next_index.get();
        let mut is_match = false;
        let found_index = self.span_starts[next_index..].iter().position(|&next_start| {
            if next_start == span.start {
                // Match. Record the match but don't exit the loop so cursor ends up on the next span.
                is_match = true;
                false
            } else {
                next_start > span.start
            }
        });

        if let Some(found_index) = found_index {
            next_index += found_index;
            self.next_index.set(next_index);
            self.next_start.set(self.span_starts[next_index]);
        } else {
            // All annotations exhausted. Set `next_start` to `u32::MAX`,
            // so that `span.start < next_start` in `has_annotation` will always be `true`.
            // No need to alter `next_index` as we can't end up in this function again,
            // so its value is irrelevant.
            self.next_start.set(u32::MAX);
        }

        is_match
    }
}
