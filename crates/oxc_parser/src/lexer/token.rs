//! Token

use oxc_span::Span;

use super::kind::Kind;

#[derive(Debug, Clone, Copy, Default)]
pub struct Token<'a> {
    /// Token Kind
    pub kind: Kind,

    /// Start offset in source
    pub start: u32,

    /// End offset in source
    pub end: u32,

    /// Indicates the token is on a newline
    pub is_on_new_line: bool,

    /// Is the original string escaped?
    pub escaped: bool,

    pub value: TokenValue<'a>,
}

#[cfg(target_pointer_width = "64")]
mod size_asserts {
    oxc_index::assert_eq_size!(super::Token, [u8; 32]);
}

impl<'a> Token<'a> {
    pub fn span(&self) -> Span {
        Span::new(self.start, self.end)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TokenValue<'a> {
    None,
    String(&'a str),
}

impl<'a> Default for TokenValue<'a> {
    fn default() -> Self {
        Self::None
    }
}

impl<'a> TokenValue<'a> {
    pub fn get_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            Self::None => None,
        }
    }
}
