use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
    ops::Deref,
};

use super::TextRange;

/// Reference to the text of a SyntaxToken without having to worry about the lifetime of `&str`.
#[derive(Eq, Clone, PartialEq)]
pub struct TokenText<'a> {
    string: Cow<'a, str>,

    /// Relative range of the "selected" token text.
    range: TextRange,
}

impl<'a> TokenText<'a> {
    pub fn new(string: impl Into<Cow<'a, str>>, range: TextRange) -> Self {
        Self { string: string.into(), range }
    }

    fn text(&self) -> &str {
        &self.string
    }
}

impl Deref for TokenText<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.text()
    }
}

impl Display for TokenText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self.text(), f)
    }
}

impl Debug for TokenText<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(self.text(), f)
    }
}
