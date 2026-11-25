use std::fmt;

use crate::SourceCode;

/// Utility struct for when you have a regular [`SourceCode`] type that doesn't
/// implement `name`. For example [`String`]. Or if you want to override the
/// `name` returned by the `SourceCode`.
///
/// `N` is the type of the name, borrowed (e.g. `&str`) or owned (e.g. [`String`]).
#[derive(Clone)]
pub struct NamedSource<S: SourceCode, N = String> {
    source: S,
    name: N,
}

impl<S: SourceCode, N: AsRef<str>> fmt::Debug for NamedSource<S, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NamedSource")
            .field("name", &self.name.as_ref())
            .field("source", &"<redacted>")
            .finish()
    }
}

impl<S: SourceCode, N: AsRef<str>> NamedSource<S, N> {
    /// Create a new `NamedSource` using a regular [`SourceCode`] and giving it
    /// a name.
    #[must_use]
    pub fn new(name: N, source: S) -> Self {
        Self { source, name }
    }
}

impl<S: SourceCode, N: AsRef<str> + Send + Sync> SourceCode for NamedSource<S, N> {
    fn data(&self) -> &[u8] {
        self.source.data()
    }

    fn name(&self) -> Option<&str> {
        Some(self.name.as_ref())
    }
}
