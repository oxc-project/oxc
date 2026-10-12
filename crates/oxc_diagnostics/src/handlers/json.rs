use std::fmt::{self, Write};

use crate::{Severity, protocol::Diagnostic, source_impls::SpanScanner};

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
        let mut scanner =
            diagnostic.source_code().map(|source| SpanScanner::new(source.data(), 0, 0));
        self.render_report_with_scanner(f, diagnostic, scanner.as_mut())
    }

    /// Render [`Diagnostic`]s in order, separated by `separator`, reusing line indexes for shared sources.
    ///
    /// # Errors
    ///
    /// Returns an error when writing the rendered reports fails.
    pub fn render_reports<'a>(
        &self,
        f: &mut impl fmt::Write,
        diagnostics: impl IntoIterator<Item = &'a dyn Diagnostic>,
        separator: &str,
    ) -> fmt::Result {
        let mut scanner: Option<SpanScanner<'a>> = None;
        for (index, diagnostic) in diagnostics.into_iter().enumerate() {
            if index > 0 {
                f.write_str(separator)?;
            }
            match diagnostic.source_code() {
                Some(source)
                    if scanner.as_ref().is_some_and(|scanner| scanner.is_for(source.data())) => {}
                Some(source) => scanner = Some(SpanScanner::new(source.data(), 0, 0)),
                None => scanner = None,
            }
            self.render_report_with_scanner(f, diagnostic, scanner.as_mut())?;
        }
        Ok(())
    }

    #[expect(clippy::unused_self, reason = "keeps a consistent renderer API")]
    fn render_report_with_scanner(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &dyn Diagnostic,
        mut scanner: Option<&mut SpanScanner<'_>>,
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

            if let Some(location) =
                scanner.as_deref_mut().and_then(|scanner| scanner.read_span(label.span()))
            {
                write!(f, r#""line": {},"#, location.line() + 1)?;
                write!(f, r#""column": {}"#, location.column() + 1)?;
            } else {
                f.write_str(r#""line": null,"column": null"#)?;
            }

            f.write_str("}}")?;
        }
        f.write_str("]}")
    }
}

#[test]
fn test_escape() {
    assert_eq!(escape("a\nb").to_string(), r"a\nb");
    assert_eq!(escape("C:\\Oxc").to_string(), r"C:\\Oxc");
}
