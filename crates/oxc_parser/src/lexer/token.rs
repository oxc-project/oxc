//! Token

use oxc_span::Span;

use super::kind::Kind;

#[derive(Debug, Clone, Copy, Default)]
pub struct Token {
    /// Token Kind
    pub kind: Kind,

    /// Start offset in source
    pub start: u32,

    /// End offset in source
    pub end: u32,

    /// Indicates the token is on a newline
    pub is_on_new_line: bool,

    /// True if the identifier / string / template kinds has escaped strings.
    /// The escaped strings are saved in [Lexer::escaped_strings] and [Lexer::escaped_templates] by
    /// [Token::start].
    ///
    /// [Lexer::escaped_strings]: [super::Lexer::escaped_strings]
    /// [Lexer::escaped_templates]: [super::Lexer::escaped_templates]
    pub escaped: bool,

    /// True if a string contains lone surrogates.
    pub lone_surrogates: bool,

    /// True if for numeric literal tokens that contain separator characters (`_`).
    ///
    /// Numeric literals are defined in Section 12.9.3 of the ECMAScript
    /// standard and include [`Kind::Decimal`], [`Kind::Binary`],
    /// [`Kind::Octal`], [`Kind::Hex`], etc.
    has_separator: bool,

    // Padding to fill to 16 bytes.
    // This makes copying a `Token` 1 x xmmword load & store, rather than 1 x dword + 1 x qword
    // and `Token::default()` is 1 x xmmword store, rather than 1 x dword + 1 x qword.
    _padding2: u16,
}

impl Token {
    pub(super) fn new_on_new_line() -> Self {
        Self { is_on_new_line: true, ..Self::default() }
    }

    pub fn span(&self) -> Span {
        Span::new(self.start, self.end)
    }

    pub fn escaped(&self) -> bool {
        self.escaped
    }

    #[inline]
    pub fn has_separator(&self) -> bool {
        debug_assert!(!self.has_separator || self.kind.is_number());
        self.has_separator
    }

    pub(crate) fn set_has_separator(&mut self) {
        debug_assert!(!self.has_separator || self.kind.is_number() || self.kind == Kind::default());
        self.has_separator = true;
    }
}

#[cfg(test)]
mod size_asserts {
    use super::Token;
    const _: () = assert!(std::mem::size_of::<Token>() == 16);
}
