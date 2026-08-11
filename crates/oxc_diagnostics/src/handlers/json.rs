use std::fmt::{self, Write};

use crate::{Severity, protocol::Diagnostic, source_impls::SpanScanner};

/**
Renders diagnostics as machine-readable JSON.
*/
#[derive(Debug, Clone)]
pub struct JSONReportHandler;

impl JSONReportHandler {
    /// Create a new [`JSONReportHandler`]. There are no customization
    /// options.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for JSONReportHandler {
    fn default() -> Self {
        Self::new()
    }
}

struct Escape<'a>(&'a str);

impl fmt::Display for Escape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in self.0.chars() {
            let escape = match c {
                '\\' => Some(r"\\"),
                '"' => Some(r#"\""#),
                '\r' => Some(r"\r"),
                '\n' => Some(r"\n"),
                '\t' => Some(r"\t"),
                '\u{08}' => Some(r"\b"),
                '\u{0c}' => Some(r"\f"),
                _ => None,
            };
            if let Some(escape) = escape {
                f.write_str(escape)?;
            } else {
                f.write_char(c)?;
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
    #[expect(clippy::unused_self, reason = "keeps a consistent renderer API")]
    pub fn render_report(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &dyn Diagnostic,
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
        write!(f, r#""severity": "{severity:}","#)?;
        if let Some(url) = diagnostic.url() {
            write!(f, r#""url": "{url}","#)?;
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
        {
            write!(f, r#""labels": ["#)?;
            let mut scanner = source.map(|source| SpanScanner::new(source.data(), 0, 0));
            let mut add_comma = false;
            for label in diagnostic.labels() {
                if add_comma {
                    write!(f, ",")?;
                } else {
                    add_comma = true;
                }
                write!(f, "{{")?;
                if let Some(label_name) = label.label() {
                    write!(f, r#""label": "{}","#, escape(label_name))?;
                }
                write!(f, r#""span": {{"#)?;
                write!(f, r#""offset": {},"#, label.offset())?;
                write!(f, r#""length": {},"#, label.len())?;

                if let Some(location) =
                    scanner.as_mut().and_then(|scanner| scanner.read_span(label.span()))
                {
                    write!(f, r#""line": {},"#, location.line() + 1)?;
                    write!(f, r#""column": {}"#, location.column() + 1)?;
                } else {
                    write!(f, r#""line": null,"column": null"#)?;
                }

                write!(f, "}}}}")?;
            }
            write!(f, "]")?;
        }
        write!(f, "}}")
    }
}

#[test]
fn test_escape() {
    assert_eq!(escape("a\nb").to_string(), r"a\nb");
    assert_eq!(escape("C:\\Oxc").to_string(), r"C:\\Oxc");
}
