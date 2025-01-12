use std::{
    borrow::Cow,
    io::{BufWriter, Stdout, Write},
};

use oxc_diagnostics::{
    reporter::{DiagnosticReporter, Info},
    Error, Severity,
};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct UnixOutputFormatter;

impl InternalFormatter for UnixOutputFormatter {
    fn all_rules(&mut self, writer: &mut BufWriter<Stdout>) {
        writeln!(writer, "flag --rules with flag --format=unix is not allowed").unwrap();
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(UnixReporter::default())
    }
}

#[derive(Default)]
struct UnixReporter {
    total: usize,
}

impl DiagnosticReporter for UnixReporter {
    fn finish(&mut self, writer: &mut dyn Write) {
        let total = self.total;
        if total > 0 {
            let line = format!("\n{total} problem{}\n", if total > 1 { "s" } else { "" });
            writer.write_all(line.as_bytes()).unwrap();
        }
        writer.flush().unwrap();
    }

    fn render_diagnostics(&mut self, writer: &mut BufWriter<Stdout>, s: &[u8]) {
        writer.write_all(s).unwrap();
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.total += 1;
        Some(format_unix(&error))
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/ae1fd9748596447d1fd09625c33d9e7ba9a3d06d/packages/eslint-formatter-unix>
fn format_unix(diagnostic: &Error) -> String {
    let Info { line, column, filename, message, severity, rule_id } = Info::new(diagnostic);
    let severity = match severity {
        Severity::Error => "Error",
        _ => "Warning",
    };
    let rule_id =
        rule_id.map_or_else(|| Cow::Borrowed(""), |rule_id| Cow::Owned(format!("/{rule_id}")));
    format!("{filename}:{line}:{column}: {message} [{severity}{rule_id}]\n")
}
