use oxc_span::Span;

pub struct SpanFactory {
    span_offset: u32,
}

impl SpanFactory {
    pub fn new(span_offset: u32) -> Self {
        Self { span_offset }
    }

    /// Add base offset to [`Span`].
    ///
    /// `Span { start: 4, end: 12 }` with `span_offset = N` →
    /// `Span { start: 4 + N, end: 12 + N }`.
    ///
    /// On valid input this never overflows: the parser rejects sources larger
    /// than `u32::MAX` bytes before any span is created, so `start`/`end` plus
    /// `span_offset` always fit in `u32`. The `debug_assert!` documents that
    /// invariant and, in debug builds, surfaces a violation loudly with context
    /// instead of letting it wrap silently. Release behavior is unchanged.
    pub fn create(&self, start: u32, end: u32) -> Span {
        debug_assert!(
            start.checked_add(self.span_offset).is_some()
                && end.checked_add(self.span_offset).is_some(),
            "SpanFactory::create overflow: span_offset={} pushes start={start}/end={end} past u32::MAX",
            self.span_offset,
        );
        Span::new(start + self.span_offset, end + self.span_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_offset_applied() {
        let factory = SpanFactory::new(10);
        let span = factory.create(4, 12);
        assert_eq!(span.start, 14);
        assert_eq!(span.end, 22);
    }

    #[test]
    fn zero_offset_is_identity() {
        let factory = SpanFactory::new(0);
        let span = factory.create(4, 12);
        assert_eq!(span.start, 4);
        assert_eq!(span.end, 12);
    }

    // The invariant guard fires loudly in debug builds rather than wrapping
    // silently. (Release builds retain the upstream wrapping behavior.)
    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "SpanFactory::create overflow")]
    fn overflowing_offset_panics_in_debug() {
        let factory = SpanFactory::new(u32::MAX);
        let _ = factory.create(1, 2);
    }
}
