use std::{
    fmt,
    sync::{Mutex, OnceLock},
};

use crate::{SourceCode, source_impls};

/// Utility struct for when you have a regular [`SourceCode`] type that doesn't
/// implement `name`. For example [`String`]. Or if you want to override the
/// `name` returned by the `SourceCode`.
pub struct NamedSource<S: SourceCode> {
    source: S,
    name: String,
    /// Complete index, built only when explicitly requested.
    line_starts: OnceLock<Option<Box<[u32]>>>,
    /// Position queries retain only the line starts scanned so far.
    line_index: Mutex<source_impls::SourceLineIndex>,
}

impl<S: SourceCode + Clone> Clone for NamedSource<S> {
    fn clone(&self) -> Self {
        Self {
            source: self.source.clone(),
            name: self.name.clone(),
            line_starts: self.line_starts.clone(),
            line_index: Mutex::new(
                self.line_index.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone(),
            ),
        }
    }
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
        Self {
            source,
            name: name.as_ref().to_string(),
            line_starts: OnceLock::new(),
            line_index: Mutex::default(),
        }
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

    fn line_column(&self, offset: usize) -> Option<(usize, usize)> {
        let input = self.source.data();
        if let Some(line_starts) = self.line_starts.get().and_then(Option::as_deref) {
            return source_impls::line_column(input, Some(line_starts), offset);
        }
        if u32::try_from(input.len()).is_err() {
            return source_impls::line_column(input, None, offset);
        }
        self.line_index
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .line_column(input, offset)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oxc_span::Span;

    use crate::{OxcDiagnostic, reporter::Info};

    use super::*;

    #[test]
    fn early_diagnostic_does_not_initialize_complete_index() {
        let source = Arc::new(NamedSource::new("file.js", "line\n".repeat(200_000)));
        let diagnostic = OxcDiagnostic::error("test")
            .with_label(Span::new(0, 1))
            .with_source_code(Arc::clone(&source));
        let info = Info::new(&diagnostic);
        assert_eq!((info.start.line, info.start.column), (1, 1));
        assert_eq!((info.end.line, info.end.column), (1, 2));
        assert!(source.line_starts.get().is_none());
    }

    #[test]
    fn cloned_and_shared_indexes_match_direct_scan() {
        let text = "first\r\nsecond\n\u{00e9}\r\u{1f980}\n";
        let source = NamedSource::new("file.js", text);
        source.line_column(8).unwrap();
        let cloned = Arc::new(source.clone());
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    for offset in (0..=text.len() + 1).rev() {
                        assert_eq!(cloned.line_column(offset), text.line_column(offset));
                    }
                });
            }
        });
        assert!(source.line_starts.get().is_none());
        cloned.line_starts().unwrap();
        for offset in 0..=text.len() + 1 {
            assert_eq!(source.line_column(offset), cloned.line_column(offset));
        }
    }
}
