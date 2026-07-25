use std::{borrow::Cow, error::Error as StdError, fmt, path::Path};

use miette::{
    Diagnostic, Labels, MietteError, MietteSpanContents, Related, SourceCode, SourceSpan,
    SpanContents,
};

use crate::{Error, service::to_file_url};

/// Convert a diagnostic source name into an absolute file URL.
///
/// Returns [`None`] if the name is already a file URL or cannot be converted.
pub fn file_url(filename: &str, cwd: &Path) -> Option<String> {
    if filename.starts_with("file:") {
        return None;
    }

    let path = Path::new(filename);
    let path = if path.is_absolute() { Cow::Borrowed(path) } else { Cow::Owned(cwd.join(path)) };
    to_file_url(path)
}

/// Replace a diagnostic's source name with an absolute file URL.
///
/// Returns the diagnostic unchanged if it has no source name or the source name is already a URL.
pub fn with_file_url(error: Error, cwd: &Path) -> Error {
    let Some(filename) = error.source_code().and_then(SourceCode::name) else {
        return error;
    };
    let Some(filename) = file_url(filename, cwd) else {
        return error;
    };

    Error::new(FileUrlDiagnostic { diagnostic: error, filename })
}

#[derive(Debug)]
struct FileUrlDiagnostic {
    diagnostic: Error,
    filename: String,
}

impl fmt::Display for FileUrlDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.diagnostic, f)
    }
}

impl StdError for FileUrlDiagnostic {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.diagnostic.source()
    }
}

impl Diagnostic for FileUrlDiagnostic {
    fn code(&self) -> Option<Cow<'_, str>> {
        self.diagnostic.code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.diagnostic.severity()
    }

    fn help(&self) -> Option<Cow<'_, str>> {
        self.diagnostic.help()
    }

    fn note(&self) -> Option<Cow<'_, str>> {
        self.diagnostic.note()
    }

    fn url(&self) -> Option<Cow<'_, str>> {
        self.diagnostic.url()
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(self)
    }

    fn labels(&self) -> Labels {
        self.diagnostic.labels()
    }

    fn related(&self) -> Related<'_> {
        self.diagnostic.related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.diagnostic.diagnostic_source()
    }
}

impl SourceCode for FileUrlDiagnostic {
    fn read_span<'a>(
        &'a self,
        span: &SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<MietteSpanContents<'a>, MietteError> {
        let source = self.diagnostic.source_code().expect("diagnostic source should exist");
        let contents = source.read_span(span, context_lines_before, context_lines_after)?;
        let language = contents.language().map(ToOwned::to_owned);
        let mut contents = MietteSpanContents::new_named(
            Cow::Borrowed(&self.filename),
            contents.data(),
            *contents.span(),
            contents.line(),
            contents.column(),
            contents.line_count(),
        );
        if let Some(language) = language {
            contents = contents.with_language(language);
        }
        Ok(contents)
    }

    fn name(&self) -> Option<&str> {
        Some(&self.filename)
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, sync::Arc};

    use miette::SourceCode;

    use crate::{NamedSource, OxcDiagnostic};

    use super::{to_file_url, with_file_url};

    #[test]
    fn replaces_relative_source_name_with_file_url() {
        let cwd = PathBuf::from(if cfg!(windows) { r"C:\project" } else { "/project" });
        let diagnostic = OxcDiagnostic::error("test")
            .with_source_code(Arc::new(NamedSource::new("src/test file.js", "test")));

        let diagnostic = with_file_url(diagnostic, &cwd);
        let name = diagnostic.source_code().and_then(SourceCode::name).unwrap();

        assert_eq!(name, to_file_url(cwd.join("src/test file.js")).unwrap());
    }
}
