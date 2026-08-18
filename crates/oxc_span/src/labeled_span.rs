use crate::Span;

/// A labeled source [`Span`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabeledSpan {
    label: Option<String>,
    span: Span,
    primary: bool,
}

impl LabeledSpan {
    /// Creates a labeled span.
    #[must_use]
    pub const fn new(label: Option<String>, offset: u32, len: u32) -> Self {
        Self { label, span: Span::sized(offset, len), primary: false }
    }

    /// Creates a labeled span from an existing span.
    #[must_use]
    pub fn new_with_span(label: Option<String>, span: impl Into<Span>) -> Self {
        Self { label, span: span.into(), primary: false }
    }

    /// Creates a primary labeled span from an existing span.
    #[must_use]
    pub fn new_primary_with_span(label: Option<String>, span: impl Into<Span>) -> Self {
        Self { label, span: span.into(), primary: true }
    }

    /// Changes the offset of the span while preserving its length.
    pub fn set_span_offset(&mut self, offset: u32) {
        self.span = Span::sized(offset, self.span.size());
    }

    /// Creates a label for the specified span.
    ///
    /// # Examples
    /// ```
    /// use oxc_span::LabeledSpan;
    ///
    /// let label = LabeledSpan::at(0..3, "should be Rust");
    /// assert_eq!(
    ///     label,
    ///     LabeledSpan::new(Some("should be Rust".to_string()), 0, 3)
    /// )
    /// ```
    #[must_use]
    pub fn at(span: impl Into<Span>, label: impl Into<String>) -> Self {
        Self::new_with_span(Some(label.into()), span)
    }

    /// Creates an unlabeled underline for the specified span.
    ///
    /// # Examples
    /// ```
    /// use oxc_span::LabeledSpan;
    ///
    /// let label = LabeledSpan::underline(12..16);
    /// assert_eq!(label, LabeledSpan::new(None, 12, 4))
    /// ```
    #[must_use]
    pub fn underline(span: impl Into<Span>) -> Self {
        Self::new_with_span(None, span)
    }

    /// Returns the label text, if present.
    #[must_use]
    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    /// Returns the source [`Span`].
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// Returns the 0-based starting byte offset.
    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.span.start
    }

    /// Returns the number of bytes this `LabeledSpan` spans.
    #[must_use]
    pub const fn len(&self) -> u32 {
        self.span.size()
    }

    /// True if this `LabeledSpan` is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.span.is_empty()
    }

    /// True if this `LabeledSpan` is a primary span.
    #[must_use]
    pub const fn primary(&self) -> bool {
        self.primary
    }
}

#[cfg(test)]
mod tests {
    use super::LabeledSpan;
    use crate::Span;

    #[test]
    fn labeled_spans_preserve_label_and_primary_state() {
        let label = LabeledSpan::new_primary_with_span(Some("here".to_string()), 3..7);
        assert_eq!(label.label(), Some("here"));
        assert_eq!(label.offset(), 3);
        assert_eq!(label.len(), 4);
        assert!(label.primary());
        assert_eq!(label.span(), Span::new(3, 7));
    }

    #[test]
    fn moving_a_label_preserves_its_length() {
        let mut label = LabeledSpan::underline(3..7);
        label.set_span_offset(10);
        assert_eq!(label.span(), Span::new(10, 14));
    }
}
