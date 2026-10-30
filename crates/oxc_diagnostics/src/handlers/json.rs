use std::fmt::{self, Write};

use crate::{Error, Severity, protocol::Diagnostic, source_impls::span_locations};

/// Renders diagnostics as machine-readable JSON.
#[derive(Debug, Clone, Default)]
pub struct JSONReportHandler;

impl JSONReportHandler {
    /// Create a new [`JSONReportHandler`]. There are no customization
    /// options.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

struct Escape<'a>(&'a str);

impl fmt::Display for Escape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for character in self.0.chars() {
            match character {
                '\\' => f.write_str(r"\\")?,
                '"' => f.write_str(r#"\""#)?,
                '\r' => f.write_str(r"\r")?,
                '\n' => f.write_str(r"\n")?,
                '\t' => f.write_str(r"\t")?,
                '\u{08}' => f.write_str(r"\b")?,
                '\u{0c}' => f.write_str(r"\f")?,
                '\u{00}'..='\u{1f}' => write!(f, r"\u{:04x}", character as u32)?,
                _ => f.write_char(character)?,
            }
        }
        Ok(())
    }
}

const fn escape(input: &'_ str) -> Escape<'_> {
    Escape(input)
}

impl JSONReportHandler {
    /// Render a [`Diagnostic`].
    ///
    /// # Errors
    ///
    /// Returns an error when writing the rendered report fails.
    pub fn render_report(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &dyn Diagnostic,
    ) -> fmt::Result {
        let locations = diagnostic.source_code().map(|source| {
            span_locations(
                source.data(),
                source.cached_line_starts(),
                diagnostic.labels().iter().map(crate::LabeledSpan::span),
            )
        });
        self.render_report_with_locations(f, diagnostic, locations.as_deref().unwrap_or_default())
    }

    /// Render diagnostics as comma-separated JSON array elements, scanning each consecutive group
    /// of diagnostics with a shared source only up to its last label.
    ///
    /// Consumes all diagnostics, including when writing fails.
    ///
    /// # Errors
    ///
    /// Returns an error when writing the rendered reports fails.
    pub fn render_reports(
        &self,
        f: &mut impl fmt::Write,
        diagnostics: &mut Vec<Error>,
    ) -> fmt::Result {
        let mut diagnostics = diagnostics.drain(..);
        while let Some(first) = diagnostics.as_slice().first() {
            let source = first.source_code();
            let end = diagnostics.as_slice()[1..]
                .iter()
                .position(|diagnostic| match (source, diagnostic.source_code()) {
                    (Some(source), Some(next)) => !std::ptr::eq(source.data(), next.data()),
                    (None, None) => false,
                    _ => true,
                })
                .map_or(diagnostics.len(), |index| index + 1);
            let mut locations = source
                .map(|source| {
                    span_locations(
                        source.data(),
                        source.cached_line_starts(),
                        diagnostics.as_slice()[..end]
                            .iter()
                            .flat_map(|diagnostic| diagnostic.labels())
                            .map(crate::LabeledSpan::span),
                    )
                })
                .unwrap_or_default()
                .into_iter();
            for (index, diagnostic) in diagnostics.by_ref().take(end).enumerate() {
                if index > 0 {
                    f.write_str(",\n")?;
                }
                let label_count = diagnostic.labels().len();
                self.render_report_with_locations(
                    f,
                    diagnostic.as_ref(),
                    locations.as_slice().get(..label_count).unwrap_or_default(),
                )?;
                for _ in 0..label_count {
                    locations.next();
                }
            }
            if !diagnostics.as_slice().is_empty() {
                f.write_str(",\n")?;
            }
        }
        Ok(())
    }

    #[expect(clippy::unused_self, reason = "keeps a consistent renderer API")]
    fn render_report_with_locations(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &dyn Diagnostic,
        locations: &[Option<(usize, usize)>],
    ) -> fmt::Result {
        write!(f, r#"{{"message": "{}","#, escape(&diagnostic.to_string()))?;
        if let Some(code) = diagnostic.code() {
            write!(f, r#""code": "{}","#, escape(&code))?;
        }
        let severity = match diagnostic.severity() {
            Some(Severity::Error) | None => "error",
            Some(Severity::Warning) => "warning",
            Some(Severity::Advice) => "advice",
        };
        write!(f, r#""severity": "{severity}","#)?;
        if let Some(url) = diagnostic.url() {
            write!(f, r#""url": "{}","#, escape(&url))?;
        }
        if let Some(help) = diagnostic.help() {
            write!(f, r#""help": "{}","#, escape(&help))?;
        }
        if let Some(note) = diagnostic.note() {
            write!(f, r#""note": "{}","#, escape(&note))?;
        }
        let source = diagnostic.source_code();
        if let Some(source) = source {
            write!(f, r#""filename": "{}","#, escape(source.name().unwrap_or_default()))?;
        }
        f.write_str(r#""labels": ["#)?;
        for (index, label) in diagnostic.labels().iter().enumerate() {
            if index > 0 {
                f.write_char(',')?;
            }
            f.write_char('{')?;
            if let Some(label_name) = label.label() {
                write!(f, r#""label": "{}","#, escape(label_name))?;
            }
            f.write_str(r#""span": {"#)?;
            write!(f, r#""offset": {},"#, label.offset())?;
            write!(f, r#""length": {},"#, label.len())?;

            if let Some(&(line, column)) = locations.get(index).and_then(Option::as_ref) {
                write!(f, r#""line": {},"#, line + 1)?;
                write!(f, r#""column": {}"#, column + 1)?;
            } else {
                f.write_str(r#""line": null,"column": null"#)?;
            }

            f.write_str("}}")?;
        }
        f.write_str("]}")
    }
}

#[test]
fn unlabelled_diagnostic_does_not_request_line_starts() {
    use crate::{OxcDiagnostic, SourceCode};

    struct Source;

    impl SourceCode for Source {
        fn data(&self) -> &[u8] {
            b"source without labels"
        }

        fn line_starts(&self) -> Option<&[u32]> {
            panic!("an unlabelled diagnostic does not need a line index")
        }
    }

    let diagnostic = OxcDiagnostic::warn("unlabelled").with_source_code(Source);
    let mut output = String::new();
    JSONReportHandler::new().render_report(&mut output, diagnostic.as_ref()).unwrap();
    assert!(output.ends_with(r#""labels": []}"#));
}

#[test]
fn batch_rendering_matches_individual_reports_without_building_a_full_index() {
    use std::sync::Arc;

    use oxc_span::Span;

    use crate::{NamedSource, OxcDiagnostic, SourceCode};

    let source = Arc::new(NamedSource::new("test.js", "first();\r\né();\nthird();\rfourth();\n"));
    let other = Arc::new(NamedSource::new("other.js", "other();\n"));
    let make_diagnostics = || {
        vec![
            OxcDiagnostic::warn("last")
                .with_label(Span::new(24, 30))
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::warn("unlabelled").with_source_code(Arc::clone(&source)),
            OxcDiagnostic::warn("reversed labels")
                .with_labels([Span::new(15, 23), Span::new(0, 5), Span::new(10, 14)])
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::warn("past EOF")
                .with_label(Span::sized(100, 1))
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::warn("no source").with_label(Span::new(0, 1)).into(),
            OxcDiagnostic::warn("other")
                .with_label(Span::new(0, 5))
                .with_source_code(Arc::clone(&other)),
            OxcDiagnostic::warn("first again")
                .with_label(Span::new(0, 5))
                .with_source_code(Arc::clone(&source)),
        ]
    };
    let handler = JSONReportHandler::new();
    let mut expected = String::new();
    for (index, diagnostic) in make_diagnostics().iter().enumerate() {
        if index > 0 {
            expected.push_str(",\n");
        }
        handler.render_report(&mut expected, diagnostic.as_ref()).unwrap();
        assert!(source.cached_line_starts().is_none());
        assert!(other.cached_line_starts().is_none());
    }
    let mut actual = String::new();
    handler.render_reports(&mut actual, &mut make_diagnostics()).unwrap();
    assert_eq!(actual, expected);
    assert!(source.cached_line_starts().is_none());
    assert!(other.cached_line_starts().is_none());

    // Already-built indexes remain reusable without changing the rendered output.
    source.line_starts().unwrap();
    other.line_starts().unwrap();
    actual.clear();
    handler.render_reports(&mut actual, &mut make_diagnostics()).unwrap();
    assert_eq!(actual, expected);
    actual.clear();
    handler.render_reports(&mut actual, &mut Vec::new()).unwrap();
    assert_eq!(actual, "");
}

#[test]
fn test_escape() {
    assert_eq!(escape("a\nb").to_string(), r"a\nb");
    assert_eq!(escape("C:\\Oxc").to_string(), r"C:\\Oxc");
}
