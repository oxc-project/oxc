use std::io::Write;

use oxc_linter::table::RuleTable;

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
