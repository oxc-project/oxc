use oxc_span::Span;

pub struct SpanFactory {
    offset: u32,
}

impl SpanFactory {
    pub fn new(offset: u32) -> Self {
        Self { offset }
    }

    /// Add base offset to `Span`.
    /// Span { start: 4, end: 12 } => Span { start: 4 + N, end: 12 + N }
    pub fn create(&self, span: Span) -> Span {
        span.expand_right(self.offset).shrink_left(self.offset)
    }
}
