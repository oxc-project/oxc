use oxc_diagnostics::{
    Error, GraphicalReportHandler, Severity,
    reporter::{DiagnosticReporter, DiagnosticResult, Info},
};

// This reporter is used with stderr and displays diagnostics only in a graphical way.
// For stdout, we display them manually in `format.rs`.

#[derive(Debug)]
pub struct DefaultReporter {
    handler: GraphicalReportHandler,
    diagnostics: Vec<Error>,
}

impl Default for DefaultReporter {
    fn default() -> Self {
        Self { handler: GraphicalReportHandler::new(), diagnostics: Vec::new() }
    }
}

impl DiagnosticReporter for DefaultReporter {
    fn render_error(&mut self, error: Error) -> Option<String> {
        // Collect diagnostics for rendering in finish() at once
        self.diagnostics.push(error);
        None
    }

    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        let mut output = String::new();

        // Render all diagnostics (errors only, no warnings)
        for diagnostic in &self.diagnostics {
            self.handler.render_report(&mut output, diagnostic.as_ref()).unwrap();
        }

        Some(output)
    }
}

#[derive(Debug, Default)]
pub struct AgentReporter;

impl DiagnosticReporter for AgentReporter {
    fn finish(&mut self, _result: &DiagnosticResult) -> Option<String> {
        None
    }

    fn supports_minified_file_fallback(&self) -> bool {
        false
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        Some(format_agent(&error))
    }
}

fn format_agent(diagnostic: &Error) -> String {
    let Info { start, filename, .. } = Info::new(diagnostic);
    let filename = if filename.is_empty() {
        diagnostic
            .source_code()
            .and_then(miette::SourceCode::name)
            .map_or_else(|| "<unknown>".to_string(), ToString::to_string)
    } else {
        filename
    };
    let severity = match diagnostic.severity() {
        Some(Severity::Warning) => "warning",
        Some(Severity::Advice) => "advice",
        _ => "error",
    };
    let message = compact_message(&diagnostic.to_string());

    if start.line == 0 {
        format!("{filename}: {severity}: {message}\n")
    } else {
        format!("{filename}:{}:{}: {severity}: {message}\n", start.line, start.column)
    }
}

fn compact_message(message: &str) -> String {
    let mut compact = String::new();
    for word in message.split_whitespace() {
        if !compact.is_empty() {
            compact.push(' ');
        }
        compact.push_str(word);
    }
    compact
}

#[cfg(test)]
mod tests {
    use oxc_diagnostics::{NamedSource, OxcDiagnostic, reporter::DiagnosticReporter};
    use oxc_span::Span;

    use super::AgentReporter;

    #[test]
    fn agent_reporter_with_label() {
        let mut reporter = AgentReporter;
        let error = OxcDiagnostic::error("Unexpected token")
            .with_label(Span::new(0, 1))
            .with_source_code(NamedSource::new("file.js", "!"));

        assert_eq!(reporter.render_error(error).unwrap(), "file.js:1:1: error: Unexpected token\n");
    }

    #[test]
    fn agent_reporter_without_label() {
        let mut reporter = AgentReporter;
        let error = OxcDiagnostic::error("Failed to save file\npermission denied")
            .with_source_code(NamedSource::new("file.js", ""));

        assert_eq!(
            reporter.render_error(error).unwrap(),
            "file.js: error: Failed to save file permission denied\n"
        );
    }
}
