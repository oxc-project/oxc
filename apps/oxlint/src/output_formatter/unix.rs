use std::io::{BufWriter, Stdout, Write};

use oxc_diagnostics::reporter::{DiagnosticReporter, UnixReporter};

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
