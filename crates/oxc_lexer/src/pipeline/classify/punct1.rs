//! Details of single-byte punctuators.

use crate::token::TokenKind;

/// Single-byte punctuator and its [`TokenKind`].
pub(super) struct Punct1 {
    pub byte: u8,
    pub kind: TokenKind,
}

impl Punct1 {
    const fn new(c: char, kind: TokenKind) -> Self {
        assert!(c.is_ascii());
        Self { byte: c as u8, kind }
    }
}

/// Single-byte punctuators and their [`TokenKind`]s.
/// `#` maps to `Invalid` - a bare `#` is invalid on its own
/// (private names and hashbangs are resolved earlier).
///
/// Does not exist at runtime - used only at compile time and in tests.
pub(super) const PUNCT1: [Punct1; 26] = [
    Punct1::new('(', TokenKind::LParen),
    Punct1::new(')', TokenKind::RParen),
    Punct1::new('[', TokenKind::LBracket),
    Punct1::new(']', TokenKind::RBracket),
    Punct1::new('{', TokenKind::LBrace),
    Punct1::new('}', TokenKind::RBrace),
    Punct1::new(';', TokenKind::Semi),
    Punct1::new(',', TokenKind::Comma),
    Punct1::new('.', TokenKind::Dot),
    Punct1::new('<', TokenKind::Lt),
    Punct1::new('>', TokenKind::Gt),
    Punct1::new('+', TokenKind::Plus),
    Punct1::new('-', TokenKind::Minus),
    Punct1::new('*', TokenKind::Star),
    Punct1::new('/', TokenKind::Slash),
    Punct1::new('%', TokenKind::Percent),
    Punct1::new('&', TokenKind::Amp),
    Punct1::new('|', TokenKind::Pipe),
    Punct1::new('^', TokenKind::Caret),
    Punct1::new('!', TokenKind::Bang),
    Punct1::new('~', TokenKind::Tilde),
    Punct1::new('?', TokenKind::Question),
    Punct1::new(':', TokenKind::Colon),
    Punct1::new('=', TokenKind::Eq),
    Punct1::new('@', TokenKind::At),
    Punct1::new('#', TokenKind::Invalid),
];
