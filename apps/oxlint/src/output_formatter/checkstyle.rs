use std::io::{BufWriter, Stdout, Write};

use oxc_diagnostics::reporter::{CheckstyleReporter, DiagnosticReporter};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct CheckStyleOutputFormatter;

impl InternalFormatter for CheckStyleOutputFormatter {
    fn all_rules(&mut self, writer: &mut BufWriter<Stdout>) {
        writeln!(writer, "flag --rules with flag --format=checkstyle is not allowed").unwrap();
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(CheckstyleReporter::default())
    }
}
