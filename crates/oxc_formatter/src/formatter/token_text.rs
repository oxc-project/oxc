use std::{fmt, ops::Deref};

use super::TextRange;

/// Reference to the text of a SyntaxToken without having to worry about the lifetime of `&str`.
#[derive(Eq, Clone, PartialEq)]
pub struct TokenText {
    // TODO: Do not allocate.
    string: String,

    /// Relative range of the "selected" token text.
    range: TextRange,
}

impl TokenText {
    pub fn new(string: String, range: TextRange) -> Self {
        Self { string, range }
    }

    fn text(&self) -> &str {
        &self.string
    }
}

impl Deref for TokenText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.text()
    }
}

impl fmt::Display for TokenText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}

impl fmt::Debug for TokenText {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.text())
    }
}
