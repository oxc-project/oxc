//! Token

use std::ops::Range;

use num_bigint::BigUint;
use oxc_ast::Atom;

use super::kind::Kind;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Token {
    /// Token Kind
    pub kind: Kind,

    /// Start offset in source
    pub start: usize,

    /// End offset in source
    pub end: usize,

    /// Indicates the token is on a newline
    pub is_on_new_line: bool,

    /// Is the original string escaped?
    pub escaped: bool,

    pub value: TokenValue,
}

impl Token {
    #[must_use]
    pub const fn range(&self) -> Range<usize> {
        self.start..self.end
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenValue {
    None,
    Number(f64),
    BigInt(BigUint),
    String(Atom),
    RegExp(RegExp),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegExp {
    pub pattern: Atom,
    pub flags: Atom,
}

impl Default for TokenValue {
    fn default() -> Self {
        Self::None
    }
}

impl TokenValue {
    #[must_use]
    pub const fn as_number(&self) -> f64 {
        match self {
            Self::Number(s) => *s,
            _ => panic!("expected number!"),
        }
    }

    #[must_use]
    pub fn as_bigint(&self) -> BigUint {
        match self {
            Self::BigInt(s) => s.clone(),
            _ => panic!("expected bigint!"),
        }
    }

    #[must_use]
    pub fn as_regex(&self) -> RegExp {
        match self {
            Self::RegExp(regex) => regex.clone(),
            _ => panic!("expected regex!"),
        }
    }

    #[must_use]
    pub const fn get_atom(&self) -> Option<&Atom> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}
