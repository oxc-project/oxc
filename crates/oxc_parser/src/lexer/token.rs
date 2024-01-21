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
}

#[cfg(target_pointer_width = "64")]
mod size_asserts {
    oxc_index::assert_eq_size!(super::Token, [u8; 12]);
}

impl Token {
    pub fn span(&self) -> Span {
        Span::new(self.start, self.end)
    }

    pub fn escaped(&self) -> bool {
        self.escaped
    }
}
