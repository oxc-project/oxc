//! Token

use oxc_ast::ast::RegExpFlags;
use oxc_span::Span;

use super::kind::Kind;

#[derive(Debug, Clone, Default)]
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
    use oxc_index::assert_eq_size;

    assert_eq_size!(super::Token, [u8; 48]);
}

impl<'a> Token<'a> {
    pub fn span(&self) -> Span {
        Span::new(self.start, self.end)
    }
}

#[derive(Debug, Clone)]
pub enum TokenValue<'a> {
    None,
    Number(f64),
    BigInt(num_bigint::BigInt),
    String(&'a str),
    RegExp(RegExp<'a>),
}

#[derive(Debug, Clone)]
pub struct RegExp<'a> {
    pub pattern: &'a str,
    pub flags: RegExpFlags,
}

impl<'a> Default for TokenValue<'a> {
    fn default() -> Self {
        Self::None
    }
}

impl<'a> TokenValue<'a> {
    pub fn as_number(&self) -> f64 {
        match self {
            Self::Number(s) => *s,
            _ => unreachable!("expected number!"),
        }
    }

    pub fn as_bigint(&self) -> num_bigint::BigInt {
        match self {
            Self::BigInt(s) => s.clone(),
            _ => unreachable!("expected bigint!"),
        }
    }

    pub fn as_regex(&self) -> &RegExp<'a> {
        match self {
            Self::RegExp(regex) => regex,
            _ => unreachable!("expected regex!"),
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}
