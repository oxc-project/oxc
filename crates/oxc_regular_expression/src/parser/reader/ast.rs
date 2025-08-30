use oxc_span::Span;

/// Represents UTF-16 code unit(u16 as u32) or Unicode code point(char as u32).
/// `Span` width may be more than 1, since there will be escape sequences.
#[derive(Debug, Clone, Copy)]
pub struct CodePoint {
    pub span: Span,
    // NOTE: If we need codegen, more information should be added.
    pub value: u32,
}
