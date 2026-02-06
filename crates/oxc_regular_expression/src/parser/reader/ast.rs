use oxc_span::Span;

/// Tracks how a code point was represented in source code.
/// This is needed to preserve information about escape sequences
/// when parsing string literals for RegExp constructor.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum EscapeKind {
    /// Not an escape sequence (literal character)
    #[default]
    None,
    /// Unicode escape: `\uXXXX` or `\u{XXXX}`
    Unicode,
    /// Hexadecimal escape: `\xXX`
    Hexadecimal,
}

/// Represents UTF-16 code unit(u16 as u32) or Unicode code point(char as u32).
/// `Span` width may be more than 1, since there will be escape sequences.
#[derive(Debug, Clone, Copy)]
pub struct CodePoint {
    pub span: Span,
    // NOTE: If we need codegen, more information should be added.
    pub value: u32,
    /// The kind of escape sequence used to represent this code point in source.
    pub escape_kind: EscapeKind,
}
