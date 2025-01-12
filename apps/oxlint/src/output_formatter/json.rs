use miette::JSONReportHandler;
use oxc_diagnostics::Error;
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

    pub fn diagnostics<T: Write>(writer: &mut T, diagnostics: &mut Vec<Error>) {
        let handler = JSONReportHandler::new();
        let messages = diagnostics
            .drain(..)
            .map(|error| {
                let mut output = String::from("\t");
                handler.render_report(&mut output, error.as_ref()).unwrap();
                output
            })
            .collect::<Vec<_>>()
            .join(",\n");

        writer.write_all(format!("[\n{messages}\n]").as_bytes()).unwrap();
    }
}
