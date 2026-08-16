//! Diagnostic-level rendering: everything except the source snippets.
//!
//! [`render_report`](GraphicalReportHandler::render_report) is the entry point.
//! It renders the title, hands off to
//! [`render_snippets`](GraphicalReportHandler::render_snippets), then renders
//! the help/note footer. Each block of prose is wrapped to the terminal width
//! using the shared [`wrap_options`](GraphicalReportHandler::wrap_options)
//! helper.

use std::fmt::{self, Write as _};

use owo_colors::OwoColorize;
use smallvec::SmallVec;

use super::handler::{GraphicalReportHandler, LinkStyle};
use crate::{Diagnostic, Severity, source_impls::SpanScanner};

struct TitleBuffer(SmallVec<[u8; 128]>);

impl TitleBuffer {
    fn new() -> Self {
        Self(SmallVec::new())
    }

    fn as_str(&self) -> &str {
        // SAFETY: writes only append valid UTF-8.
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl fmt::Write for TitleBuffer {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        self.0.extend_from_slice(text.as_bytes());
        Ok(())
    }
}

impl GraphicalReportHandler {
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
        let source = diagnostic.source_code();
        let mut scanner = source.map(|source| SpanScanner::new(source.data(), 1, 1));
        let source_name = source.and_then(|source| source.name());
        self.render_report_with_scanner(f, diagnostic, scanner.as_mut(), source_name)
    }

    /// Render [`Diagnostic`]s in order, reusing line indexes for shared sources.
    ///
    /// # Errors
    ///
    /// Returns an error when writing the rendered reports fails.
    pub fn render_reports<'a>(
        &self,
        f: &mut impl fmt::Write,
        diagnostics: impl IntoIterator<Item = &'a dyn Diagnostic>,
    ) -> fmt::Result {
        let mut scanner: Option<SpanScanner<'a>> = None;
        for diagnostic in diagnostics {
            let source = diagnostic.source_code();
            match source {
                Some(source)
                    if scanner.as_ref().is_some_and(|scanner| scanner.is_for(source.data())) => {}
                Some(source) => scanner = Some(SpanScanner::new(source.data(), 1, 1)),
                None => scanner = None,
            }
            let source_name = source.and_then(|source| source.name());
            self.render_report_with_scanner(f, diagnostic, scanner.as_mut(), source_name)?;
        }
        Ok(())
    }

    /// Render [`Diagnostic`]s until `keep` rejects a rendered report.
    ///
    /// # Errors
    ///
    /// Returns an error when writing the rendered reports fails.
    pub fn render_reports_until<'a>(
        &self,
        diagnostics: impl IntoIterator<Item = &'a dyn Diagnostic>,
        keep: &mut dyn FnMut(&dyn Diagnostic, &str) -> bool,
    ) -> fmt::Result {
        let mut scanner: Option<SpanScanner<'a>> = None;
        let mut output = String::new();
        for diagnostic in diagnostics {
            let source = diagnostic.source_code();
            match source {
                Some(source)
                    if scanner.as_ref().is_some_and(|scanner| scanner.is_for(source.data())) => {}
                Some(source) => scanner = Some(SpanScanner::new(source.data(), 1, 1)),
                None => scanner = None,
            }
            let source_name = source.and_then(|source| source.name());
            output.clear();
            self.render_report_with_scanner(
                &mut output,
                diagnostic,
                scanner.as_mut(),
                source_name,
            )?;
            if !keep(diagnostic, &output) {
                break;
            }
        }
        Ok(())
    }

    fn render_report_with_scanner(
        &self,
        f: &mut impl fmt::Write,
        diagnostic: &dyn Diagnostic,
        scanner: Option<&mut SpanScanner<'_>>,
        source_name: Option<&str>,
    ) -> fmt::Result {
        writeln!(f)?;
        self.render_title(f, diagnostic)?;
        self.render_snippets(f, diagnostic, scanner, source_name)?;
        self.render_footer(f, diagnostic)?;
        Ok(())
    }

    fn render_title(&self, f: &mut impl fmt::Write, diagnostic: &dyn Diagnostic) -> fmt::Result {
        let (severity_style, severity_icon) = match diagnostic.severity() {
            Some(Severity::Error) | None => (self.theme.styles.error, &self.theme.characters.error),
            Some(Severity::Warning) => (self.theme.styles.warning, &self.theme.characters.warning),
            Some(Severity::Advice) => (self.theme.styles.advice, &self.theme.characters.advice),
        };

        let width = self.termwidth.saturating_sub(2);

        let mut title = TitleBuffer::new();
        match (self.links, diagnostic.url(), diagnostic.code()) {
            (LinkStyle::Link, Some(url), Some(code)) => {
                // magic unicode escape sequences to make the terminal print a hyperlink
                const CTL: &str = "\u{1b}]8;;";
                const END: &str = "\u{1b}]8;;\u{1b}\\";
                let code = code.style(severity_style);
                let diagnostic = diagnostic.style(severity_style);
                write!(title, "{CTL}{url}\u{1b}\\{code}{END}: {diagnostic}")?;
            }
            (_, _, Some(code)) if severity_style.is_plain() => {
                write!(title, "{code}: {diagnostic}")?;
            }
            (_, _, Some(code)) => {
                write!(title, "{}", format_args!("{code}: {diagnostic}").style(severity_style))?;
            }
            _ if severity_style.is_plain() => write!(title, "{diagnostic}")?,
            _ => write!(title, "{}", diagnostic.style(severity_style))?,
        }
        let title = title.as_str();
        if !title.contains('\n')
            && severity_icon.len().saturating_add(title.len()).saturating_add(3) <= width
        {
            f.write_str("  ")?;
            if severity_style.is_plain() {
                f.write_str(severity_icon)?;
            } else {
                write!(f, "{}", severity_icon.style(severity_style))?;
            }
            f.write_char(' ')?;
            f.write_str(title.trim_end_matches(' '))?;
        } else {
            // No-color themes can bypass owo-colors' formatting machinery entirely.
            let (initial_indent, rest_indent) = if severity_style.is_plain() {
                (format!("  {severity_icon} "), format!("  {} ", self.theme.characters.vbar))
            } else {
                (
                    format!("  {} ", severity_icon.style(severity_style)),
                    format!("  {} ", self.theme.characters.vbar.style(severity_style)),
                )
            };
            let opts = Self::wrap_options(width, &initial_indent, &rest_indent);
            Self::write_fill(f, title, opts)?;
        }
        f.write_char('\n')?;

        Ok(())
    }

    fn render_footer(&self, f: &mut impl fmt::Write, diagnostic: &dyn Diagnostic) -> fmt::Result {
        if let Some(help) = diagnostic.help() {
            const PREFIX: &str = "  help: ";
            let width = self.termwidth.saturating_sub(4);
            if memchr::memchr(b'\n', help.as_bytes()).is_none()
                && PREFIX.len().saturating_add(help.len()) <= width
            {
                if self.theme.styles.help.is_plain() {
                    f.write_str(PREFIX)?;
                } else {
                    write!(f, "{}", PREFIX.style(self.theme.styles.help))?;
                }
                f.write_str(help.trim_end_matches(' '))?;
            } else {
                let initial_indent = PREFIX.style(self.theme.styles.help).to_string();
                let opts = Self::wrap_options(width, &initial_indent, "        ");
                Self::write_fill(f, &help, opts)?;
            }
            f.write_char('\n')?;
        }
        if let Some(note) = diagnostic.note() {
            // Renders as:
            //   note: This is a note about the error
            let width = self.termwidth.saturating_sub(4);
            let initial_indent = "  note: ".style(self.theme.styles.note).to_string();
            let opts = Self::wrap_options(width, &initial_indent, "           ");
            Self::write_fill(f, &note, opts)?;
            f.write_char('\n')?;
        }
        Ok(())
    }

    /// Builds the [`textwrap::Options`] shared by every wrapped block.
    fn wrap_options<'a>(
        width: usize,
        initial_indent: &'a str,
        subsequent_indent: &'a str,
    ) -> textwrap::Options<'a> {
        textwrap::Options::new(width)
            .initial_indent(initial_indent)
            .subsequent_indent(subsequent_indent)
    }

    fn write_fill(f: &mut impl fmt::Write, text: &str, opts: textwrap::Options<'_>) -> fmt::Result {
        if Self::fits_on_line(text, &opts) {
            f.write_str(opts.initial_indent)?;
            f.write_str(text.trim_end_matches(' '))
        } else {
            f.write_str(&textwrap::fill(text, opts))
        }
    }

    /// Skip word separation and optimal-fit layout when the text demonstrably
    /// fits on its first line. `textwrap` only provides this fast path without
    /// indentation, while every diagnostic block has an initial indent.
    #[cfg(test)]
    fn fill(text: &str, opts: textwrap::Options<'_>) -> String {
        if Self::fits_on_line(text, &opts) {
            let text = text.trim_end_matches(' ');
            let mut result = String::with_capacity(opts.initial_indent.len() + text.len());
            result.push_str(opts.initial_indent);
            result.push_str(text);
            return result;
        }
        textwrap::fill(text, opts)
    }

    fn fits_on_line(text: &str, opts: &textwrap::Options<'_>) -> bool {
        if memchr::memchr(b'\n', text.as_bytes()).is_some() {
            return false;
        }

        // UTF-8 byte length is an upper bound on terminal display width,
        // including for ANSI escape sequences. Avoid both width scans when
        // even that conservative bound fits.
        opts.initial_indent.len().saturating_add(text.len()) <= opts.width || {
            let available = opts.width.saturating_sub(Self::display_width(opts.initial_indent));
            Self::display_width(text) <= available
        }
    }

    /// Compute terminal width bytewise for ASCII, including the CSI and OSC
    /// escape sequences recognized by `textwrap`. Unicode retains its full
    /// width calculation.
    fn display_width(text: &str) -> usize {
        if !text.is_ascii() {
            return textwrap::core::display_width(text);
        }

        let bytes = text.as_bytes();
        let mut width = 0;
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] != b'\x1b' {
                width += usize::from((b' '..=b'~').contains(&bytes[i]));
                i += 1;
                continue;
            }

            i += 1;
            let Some(&kind) = bytes.get(i) else { break };
            i += 1;
            match kind {
                b'[' => {
                    while i < bytes.len() {
                        let byte = bytes[i];
                        i += 1;
                        if (b'@'..=b'~').contains(&byte) {
                            break;
                        }
                    }
                }
                b']' => {
                    while i < bytes.len() {
                        if bytes[i] == b'\x07' {
                            i += 1;
                            break;
                        }
                        if bytes[i] == b'\x1b' && bytes.get(i + 1) == Some(&b'\\') {
                            i += 2;
                            break;
                        }
                        i += 1;
                    }
                }
                _ => {}
            }
        }
        width
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use oxc_span::Span;

    use super::*;
    use crate::{Error, GraphicalTheme, NamedSource, OxcDiagnostic};

    #[test]
    fn batch_render_matches_individual_reports() {
        let source =
            Arc::new(NamedSource::new("input.ts", "first();\nsecond();\nthird();\nfourth();\n"));
        let other_source = Arc::new(NamedSource::new("other.ts", "alpha();\nbeta();\n"));
        let diagnostics: Vec<Error> = vec![
            OxcDiagnostic::warn("later")
                .with_label(Span::new(29, 35))
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::error("multi-label")
                .with_labels([Span::new(0, 5), Span::new(19, 24)])
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::warn("earlier")
                .with_label(Span::new(9, 15))
                .with_source_code(Arc::clone(&source)),
            OxcDiagnostic::error("without source").into(),
            OxcDiagnostic::warn("different source")
                .with_label(Span::new(9, 13))
                .with_source_code(other_source),
        ];
        let handler = GraphicalReportHandler::new_themed(GraphicalTheme::none()).with_links(false);

        let mut expected = String::new();
        for diagnostic in &diagnostics {
            handler.render_report(&mut expected, diagnostic.as_ref()).unwrap();
        }

        let mut actual = String::new();
        handler
            .render_reports(
                &mut actual,
                diagnostics.iter().map(|diagnostic| diagnostic.as_ref() as &dyn Diagnostic),
            )
            .unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    #[cfg_attr(
        miri,
        ignore = "exhaustive equivalence check over safe text wrapping code; interpreting every \
                  textwrap case under Miri takes more than 16 minutes"
    )]
    fn fill_fast_path_matches_textwrap() {
        let texts = [
            "",
            "short diagnostic",
            "trailing spaces   ",
            "  leading spaces",
            "two  inner  spaces",
            "Café 火",
            "combining e\u{301}",
            "emoji 🐂",
            "\u{1b}[31mstyled text\u{1b}[0m",
            "\u{1b}]8;;https://example.com\u{1b}\\linked\u{1b}]8;;\u{1b}\\",
            "\u{1b}]0;title\u{7}visible",
            "control\tcharacters\u{7}",
            "incomplete \u{1b}[31",
            "first\nsecond",
        ];
        for width in 0..32 {
            for initial_indent in ["", "  ", "  help: ", "\u{1b}[31m  × \u{1b}[0m"] {
                for text in texts {
                    let opts = textwrap::Options::new(width)
                        .initial_indent(initial_indent)
                        .subsequent_indent("    ");
                    assert_eq!(
                        GraphicalReportHandler::fill(text, opts.clone()),
                        textwrap::fill(text, opts.clone()),
                        "width={width}, indent={initial_indent:?}, text={text:?}"
                    );
                    let mut output = String::new();
                    GraphicalReportHandler::write_fill(&mut output, text, opts.clone()).unwrap();
                    assert_eq!(
                        output,
                        textwrap::fill(text, opts),
                        "streaming: width={width}, indent={initial_indent:?}, text={text:?}"
                    );
                }
            }
        }
    }
}
