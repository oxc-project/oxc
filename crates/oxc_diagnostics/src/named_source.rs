use std::fmt;

use crate::SourceCode;

/// Utility struct for when you have a regular [`SourceCode`] type that doesn't
/// implement `name`. For example [`String`]. Or if you want to override the
/// `name` returned by the `SourceCode`.
#[derive(Clone)]
pub struct NamedSource<S: SourceCode + 'static> {
    source: S,
    name: String,
}

impl<S: SourceCode> fmt::Debug for NamedSource<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NamedSource").field("name", &self.name).field("source", &"<redacted>");
        Ok(())
    }
}

impl<S: SourceCode + 'static> NamedSource<S> {
    /// Create a new `NamedSource` using a regular [`SourceCode`] and giving it
    /// a name.
    #[must_use]
    pub fn new(name: impl AsRef<str>, source: S) -> Self
    where
        S: Send + Sync,
    {
        Self { source, name: name.as_ref().to_string() }
    }
}

impl<S: SourceCode + 'static> SourceCode for NamedSource<S> {
    fn data(&self) -> &[u8] {
        self.source.data()
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }
}
