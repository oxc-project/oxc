use std::io::{BufWriter, Stdout, Write};

use oxc_diagnostics::reporter::{DiagnosticReporter, GraphicalReporter};
use oxc_linter::table::RuleTable;

use crate::output_formatter::InternalFormatter;

#[derive(Debug)]
pub struct DefaultOutputFormatter;

impl InternalFormatter for DefaultOutputFormatter {
    fn all_rules(&mut self, writer: &mut BufWriter<Stdout>) {
        let table = RuleTable::new();
        for section in table.sections {
            writeln!(writer, "{}", section.render_markdown_table(None)).unwrap();
        }
        writeln!(writer, "Default: {}", table.turned_on_by_default_count).unwrap();
        writeln!(writer, "Total: {}", table.total).unwrap();
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(GraphicalReporter::default())
    }
}

#[cfg(test)]
mod test {
    use crate::output_formatter::{default::DefaultOutputFormatter, InternalFormatter};
    use std::io::BufWriter;

    #[test]
    fn all_rules() {
        let mut writer = BufWriter::new(std::io::stdout());
        let mut formatter = DefaultOutputFormatter;

        formatter.all_rules(&mut writer);
        assert!(!writer.buffer().is_empty());
    }
}
