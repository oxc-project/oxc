//! All Parser / Linter Diagnostics

use std::{cell::RefCell, ops::Deref, rc::Rc};

use oxc_ast::Span;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Diagnostic>;

#[derive(Debug, Default, Clone)]
pub struct Diagnostics(Rc<RefCell<Vec<Diagnostic>>>);

impl Deref for Diagnostics {
    type Target = Rc<RefCell<Vec<Diagnostic>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Diagnostics {
    /// # Panics
    #[must_use]
    pub fn into_inner(self) -> Vec<Diagnostic> {
        Rc::try_unwrap(self.0).unwrap().into_inner()
    }
}

#[derive(Debug, Clone, Error, miette::Diagnostic)]
pub enum Diagnostic {
    #[error("This file panicked")]
    #[diagnostic()]
    Panic(#[label("")] Span),

    /* Lexer */
    #[error("Syntax Error")]
    #[diagnostic()]
    UnexpectedToken(#[label("Unexpected Token")] Span),

    #[error("Syntax Error")]
    #[diagnostic()]
    ExpectToken(&'static str, &'static str, #[label("Expect `{0}` here, but found `{1}`")] Span),

    #[error("Invalid escape sequence")]
    InvalidEscapeSequence(#[label("Invalid escape sequence")] Span),

    #[error("Invalid escape sequence")]
    NonOctalDecimalEscapeSequence(#[label("\\8 and \\9 are not allowed in strict mode")] Span),

    #[error("Invalid Unicode escape sequence")]
    UnicodeEscapeSequence(#[label("Invalid Unicode escape sequence")] Span),

    #[error("Invalid Character `{0}`")]
    InvalidCharacter(char, #[label("Invalid Character `{0}`")] Span),

    #[error("Invalid characters after number")]
    InvalidNumberEnd(#[label("Invalid characters after number")] Span),

    #[error("Unterminated multiLine comment")]
    UnterminatedMultiLineComment(#[label("Unterminated multiLine comment")] Span),

    #[error("Unterminated string")]
    UnterminatedString(#[label("Unterminated string")] Span),

    #[error("Unexpected flag {0} in regular expression literal")]
    RegExpFlag(char, #[label("Unexpected flag {0} in regular expression literal")] Span),

    #[error("Flag {0} is mentioned twice in regular expression literal")]
    RegExpFlagTwice(
        char,
        #[label("Flag {0} is mentioned twice in regular expression literal")] Span,
    ),

    #[error("The 'u' and 'v' regular expression flags cannot be enabled at the same time")]
    RegExpFlagUAndV(
        #[label("The 'u' and 'v' regular expression flags cannot be enabled at the same time")]
        Span,
    ),

    #[error("Unexpected end of file")]
    UnexpectedEnd(#[label("Unexpected end of file")] Span),

    #[error("Unterminated regular expression")]
    UnterminatedRegExp(#[label("Unterminated regular expression")] Span),

    #[error("Invalid Number")]
    InvalidNumber(&'static str, #[label("{0}")] Span),

    #[error("Keywords cannot contain escape characters")]
    #[diagnostic()]
    EscapedKeyword(#[label("keyword cannot contain escape characters")] Span),
}
