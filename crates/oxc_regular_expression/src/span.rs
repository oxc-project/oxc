use oxc_span::Span;

pub struct SpanFactory {
    span_offset: u32,
}

impl SpanFactory {
    pub fn new(span_offset: u32) -> Self {
        Self { span_offset }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn create(&self, start: usize, end: usize) -> Span {
        Span::new((start as u32) + self.span_offset, (end as u32) + self.span_offset)
    }
}
