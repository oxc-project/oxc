use std::ops::Deref;

use super::TextRange;

/// Reference to the text of a SyntaxToken without having to worry about the lifetime of `&str`.
#[derive(Debug, Eq, Clone, PartialEq)]
pub struct TokenText {
    // // Using a green token to ensure this type is Send + Sync.
    // token: GreenToken,
    /// Relative range of the "selected" token text.
    range: TextRange,
}

impl Deref for TokenText {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        ""
    }
}
