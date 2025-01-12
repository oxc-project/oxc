use std::io::{BufWriter, Stdout, Write};

use oxc_diagnostics::reporter::{DiagnosticReporter, JsonReporter};
use oxc_linter::rules::RULES;
use oxc_linter::RuleCategory;

use crate::output_formatter::InternalFormatter;

#[derive(Debug, Default)]
pub struct JsonOutputFormatter;

impl InternalFormatter for JsonOutputFormatter {
    fn all_rules(&mut self, writer: &mut BufWriter<Stdout>) {
        #[derive(Debug, serde::Serialize)]
        struct RuleInfoJson<'a> {
            scope: &'a str,
            value: &'a str,
            category: RuleCategory,
        }

        let rules_info = RULES.iter().map(|rule| RuleInfoJson {
            scope: rule.plugin_name(),
            value: rule.name(),
            category: rule.category(),
        });

        writer
            .write_all(
                serde_json::to_string_pretty(&rules_info.collect::<Vec<_>>())
                    .expect("Failed to serialize")
                    .as_bytes(),
            )
            .unwrap();
    }

    fn get_diagnostic_reporter(&self) -> Box<dyn DiagnosticReporter> {
        Box::new(JsonReporter::default())
    }
}
