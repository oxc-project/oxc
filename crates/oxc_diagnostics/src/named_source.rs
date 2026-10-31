use std::{fmt, sync::OnceLock};

use crate::{SourceCode, source_impls};

/// Utility struct for when you have a regular [`SourceCode`] type that doesn't
/// implement `name`. For example [`String`]. Or if you want to override the
/// `name` returned by the `SourceCode`.
#[derive(Clone)]
pub struct NamedSource<S: SourceCode> {
    source: S,
    name: String,
    /// Line starts of `source`, computed on first use. Reporters resolve every
    /// diagnostic's line and column through this, so a file's many diagnostics
    /// share one scan instead of each rescanning the file.
    line_starts: OnceLock<Option<Box<[u32]>>>,
}

impl<S: SourceCode> fmt::Debug for NamedSource<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NamedSource")
            .field("name", &self.name)
            .field("source", &"<redacted>")
            .finish_non_exhaustive()
    }
}

impl<S: SourceCode> NamedSource<S> {
    /// Create a new `NamedSource` using a regular [`SourceCode`] and giving it
    /// a name.
    #[must_use]
    pub fn new(name: impl AsRef<str>, source: S) -> Self {
        Self { source, name: name.as_ref().to_string(), line_starts: OnceLock::new() }
    }
}

impl<S: SourceCode> SourceCode for NamedSource<S> {
    fn data(&self) -> &[u8] {
        self.source.data()
    }

    fn name(&self) -> Option<&str> {
        Some(&self.name)
    }

    fn line_starts(&self) -> Option<&[u32]> {
        self.line_starts.get_or_init(|| source_impls::line_starts(self.source.data())).as_deref()
    }
}
