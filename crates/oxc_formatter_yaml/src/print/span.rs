use oxc_span::Span;

/// Bridges the parser's byte-offset span to [`oxc_span::Span`].
/// `pub`: also used by `format.rs` for the parser's comment spans (re-exported through `print`).
#[inline]
pub fn to_span(span: oxc_yaml_parser::Span) -> Span {
    Span::new(span.start, span.end)
}
