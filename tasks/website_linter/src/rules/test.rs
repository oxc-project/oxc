use std::sync::{Arc, OnceLock};

use markdown::{to_html_with_options, Options};
use oxc_allocator::Allocator;
use oxc_diagnostics::NamedSource;
use oxc_linter::table::RuleTable;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::render_rules_table;

static TABLE: OnceLock<RuleTable> = OnceLock::new();

fn table() -> &'static RuleTable {
    TABLE.get_or_init(RuleTable::new)
}

fn parse(filename: &str, jsx: &str) -> Result<(), String> {
    let filename = format!("{filename}.tsx");
    let source_type = SourceType::from_path(&filename).unwrap();
    parse_type(&filename, jsx, source_type)
}
