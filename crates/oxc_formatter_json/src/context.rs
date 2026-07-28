use std::cell::Cell;

use oxc_ast::Comment;
use oxc_diagnostics::OxcDiagnostic;
use oxc_formatter_core::{FormatContext, SourceText};
use oxc_span::Span;

use crate::{comments::Comments, options::JsonFormatOptions};

/// Formatting context for JSON.
pub struct JsonFormatContext<'a> {
    options: JsonFormatOptions,
    source_text: SourceText<'a>,
    comments: Comments<'a>,
    /// Byte offset within `source_text` where the user's original source begins.
    /// `1` on the wrapped parse path (skip the leading `(`), `0` on the bare fallback.
    /// Used by [`Self::report_invalid_json`] to report user-visible line/column.
    source_offset: u32,
    /// First-error slot.
    /// `Box` keeps the happy-path field size to a single word.
    /// (the Option<Box<...>> is `None` for valid JSON and never allocates).
    error: Cell<Option<Box<OxcDiagnostic>>>,
}

impl<'a> JsonFormatContext<'a> {
    pub fn new(
        options: JsonFormatOptions,
        source_code: &'a str,
        comments: &'a [Comment],
        source_offset: u32,
    ) -> Self {
        Self {
            options,
            source_text: SourceText::new(source_code),
            comments: Comments::new(comments),
            source_offset,
            error: Cell::new(None),
        }
    }

    /// Returns the source text with the arena lifetime (vs the trait's borrow-elided `&str`).
    /// Slices taken via this method (e.g. `slice_range`, `bytes_range`) carry the `'a` lifetime,
    /// so they don't have to be re-allocated for `text(...)`.
    pub fn source_text(&self) -> SourceText<'a> {
        self.source_text
    }

    /// Returns the comment cursor.
    pub fn comments(&self) -> &Comments<'a> {
        &self.comments
    }

    /// Records the first invalid-JSON occurrence, subsequent calls are ignored.
    /// Since `span` is in `source_code` (wrapped) coordinates,
    /// we adjust it by `source_offset` so the error label is in user-source coordinates.
    pub fn report_invalid_json(&self, span: Span) {
        // Peek-without-consume on `Cell<Option<Box<_>>>`: take and put back if already set.
        let existing = self.error.take();
        if existing.is_some() {
            self.error.set(existing);
            return;
        }
        let user_span = span.move_left(self.source_offset);
        self.error.set(Some(Box::new(
            OxcDiagnostic::error("This syntax is not allowed in JSON").with_label(user_span),
        )));
    }

    /// Drains and returns any recorded error.
    pub fn take_error(&self) -> Option<OxcDiagnostic> {
        self.error.take().map(|b| *b)
    }
}

impl FormatContext for JsonFormatContext<'_> {
    type Options = JsonFormatOptions;

    fn options(&self) -> &Self::Options {
        &self.options
    }

    fn source_code(&self) -> &str {
        &self.source_text
    }
}
