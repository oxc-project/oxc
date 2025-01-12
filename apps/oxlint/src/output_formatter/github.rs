use std::io::{BufWriter, Stdout, Write};

use oxc_diagnostics::reporter::{DiagnosticReporter, GithubReporter};

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct GithubOutputFormatter;

impl InternalFormatter for GithubOutputFormatter {
    fn all_rules(&mut self, writer: &mut BufWriter<Stdout>) {
        writeln!(writer, "flag --rules with flag --format=github is not allowed").unwrap();
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(GithubReporter)
    }
}
