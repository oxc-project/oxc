use std::{
    borrow::Cow,
    io::{BufWriter, Stdout, Write},
};

use super::{writer, DiagnosticReporter, Info};
use crate::{Error, Severity};

pub struct UnixReporter {
    total: usize,
    writer: BufWriter<Stdout>,
}

impl Default for UnixReporter {
    fn default() -> Self {
        Self { total: 0, writer: writer() }
    }
}

impl DiagnosticReporter for UnixReporter {
    fn finish(&mut self) {
        let total = self.total;
        if total > 0 {
            let line = format!("\n{total} problem{}\n", if total > 1 { "s" } else { "" });
            self.writer.write_all(line.as_bytes()).unwrap();
        }
        self.writer.flush().unwrap();
    }

    fn render_diagnostics(&mut self, s: &[u8]) {
        self.writer.write_all(s).unwrap();
    }

    fn render_error(&mut self, error: Error) -> Option<String> {
        self.total += 1;
        Some(format_unix(&error))
    }
}

/// <https://github.com/fregante/eslint-formatters/tree/main/packages/eslint-formatter-unix>
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
