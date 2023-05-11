//! Token

use num_bigint::BigUint;
use oxc_ast::ast::RegExpFlags;
use oxc_span::Span;

use super::kind::Kind;

#[derive(Debug, Clone, PartialEq, Default)]
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
    use oxc_index::static_assert_size;

    use super::Token;

    static_assert_size!(Token, 48);
}

impl<'a> Token<'a> {
    pub fn span(&self) -> Span {
        Span::new(self.start, self.end)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue<'a> {
    None,
    Number(f64),
    BigInt(BigUint),
    String(&'a str),
    RegExp(RegExp<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
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
            _ => panic!("expected number!"),
        }
    }

    pub fn as_bigint(&self) -> BigUint {
        match self {
            Self::BigInt(s) => s.clone(),
            _ => panic!("expected bigint!"),
        }
    }

    pub fn as_regex(&self) -> &RegExp<'a> {
        match self {
            Self::RegExp(regex) => regex,
            _ => panic!("expected regex!"),
        }
    }

    pub fn get_string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}
