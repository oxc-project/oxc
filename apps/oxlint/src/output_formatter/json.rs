use oxc_linter::rules::RULES;
use oxc_linter::RuleCategory;
use std::io::Write;

#[derive(Debug)]
pub struct JsonOutputFormatter;

impl JsonOutputFormatter {
    pub fn all_rules<T: Write>(writer: &mut T) {
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
}
