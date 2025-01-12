use std::io::Write;

use oxc_linter::table::RuleTable;
use oxc_diagnostics::{GraphicalReportHandler, Error};

pub struct DefaultOutputFormatter;

impl DefaultOutputFormatter {
    pub fn all_rules<T: Write>(writer: &mut T) {
        let table = RuleTable::new();
        for section in table.sections {
            writeln!(writer, "{}", section.render_markdown_table(None)).unwrap();
        }
        writeln!(writer, "Default: {}", table.turned_on_by_default_count).unwrap();
        writeln!(writer, "Total: {}", table.total).unwrap();
    }

    pub fn diagnostics<T: Write + std::fmt::Write>(writer: &mut T, diagnostics: &mut Vec<Error>) {
        let handler = GraphicalReportHandler::new();

        for error in diagnostics {
            handler.render_report( writer, error.as_ref()).unwrap();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::output_formatter::default::DefaultOutputFormatter;

    #[test]
    fn all_rules() {
        let mut writer = Vec::new();

        DefaultOutputFormatter::all_rules(&mut writer);
        assert!(!writer.is_empty());
    } 
}
