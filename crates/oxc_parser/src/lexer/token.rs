//! Token

use oxc_span::Span;

use super::kind::Kind;

pub type EscapedId = std::num::NonZeroU32;

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

    /// A index handle to `Lexer::escaped_strings` or `Lexer::escaped_templates`
    /// See https://floooh.github.io/2018/06/17/handles-vs-pointers.html for some background reading
    pub escaped_id: Option<EscapedId>,
}

#[cfg(target_pointer_width = "64")]
mod size_asserts {
    oxc_index::assert_eq_size!(super::Token, [u8; 16]);
}

impl Token {
    pub fn span(&self) -> Span {
        Span::new(self.start, self.end)
    }

    pub fn escaped(&self) -> bool {
        self.escaped_id.is_some()
    }
}
