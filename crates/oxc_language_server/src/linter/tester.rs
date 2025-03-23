use oxc_linter::Linter;
use tower_lsp::lsp_types::Url;

use super::{error_with_position::DiagnosticReport, server_linter::ServerLinter};

const CURRENT_DIR: &str = env!("CARGO_MANIFEST_DIR");

/// Given a file path relative to the crate root directory, return the URI of the file.
pub fn get_file_uri(relative_file_path: &str) -> Url {
    let absolute_file_path = PathBuf::from(CURRENT_DIR).join(relative_file_path);
    let file_path = format!("file://{}", absolute_file_path.display());
    Url::parse(&file_path).unwrap()
}

use std::path::PathBuf;

use cow_utils::CowUtils;
use tower_lsp::lsp_types::{CodeDescription, NumberOrString};

fn get_snapshot_from_report(report: &DiagnosticReport) -> String {
    let code = match &report.diagnostic.code {
        Some(NumberOrString::Number(code)) => code.to_string(),
        Some(NumberOrString::String(code)) => code.to_string(),
        None => "None".to_string(),
    };
    let code_description_href = match &report.diagnostic.code_description {
        Some(CodeDescription { href }) => href.to_string(),
        None => "None".to_string(),
    };
    let message = report.diagnostic.message.clone();
    let range = report.diagnostic.range;
    let related_information = match &report.diagnostic.related_information {
        Some(infos) => {
            infos
                .iter()
                .enumerate()
                .map(|(i, info)| {
                    let mut result = String::new();
                    result.push_str(&format!(
                        "related_information[{}].message: {:?}",
                        i, info.message
                    ));
                    result.push_str(&format!(
                        "\nrelated_information[{}].location.uri: {:?}",
                        i,
                        // replace everything between `file://` and crate dir with `<variable>`, to avoid
                        // the absolute path causing snapshot test failures
                        info.location
                            .uri
                            .to_string()
                            .cow_replace(&format!("{CURRENT_DIR}/"), "<variable>/")
                    ));
                    result.push_str(&format!(
                        "\nrelated_information[{}].location.range: {:?}",
                        i, info.location.range
                    ));
                    result
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        None => "related_information: None".to_string(),
    };
    let severity = report.diagnostic.severity;
    let source = report.diagnostic.source.clone();
    let tags = report.diagnostic.tags.clone();
    format!(
        r"
code: {code:?}
code_description.href: {code_description_href:?}
message: {message:?}
range: {range:?}
{related_information}
severity: {severity:?}
source: {source:?}
tags: {tags:?}
            "
    )
}

/// Testing struct for the [linter server][crate::linter::server_linter::ServerLinter].
pub struct Tester<'t> {
    server_linter: ServerLinter,
    snapshot_suffix: Option<&'t str>,
}

impl Tester<'_> {
    pub fn new() -> Self {
        Self { snapshot_suffix: None, server_linter: ServerLinter::new() }
    }

    pub fn new_with_linter(linter: Linter) -> Self {
        Self { snapshot_suffix: None, server_linter: ServerLinter::new_with_linter(linter) }
    }

    pub fn with_snapshot_suffix(mut self, suffix: &'static str) -> Self {
        self.snapshot_suffix = Some(suffix);
        self
    }

    /// Given a relative file path (relative to `oxc_language_server` crate root), run the linter
    /// and return the resulting diagnostics in a custom snapshot format.
    #[expect(clippy::disallowed_methods)]
    pub fn test_and_snapshot_single_file(&self, relative_file_path: &str) {
        let uri = get_file_uri(relative_file_path);
        let content = std::fs::read_to_string(uri.to_file_path().unwrap())
            .expect("could not read fixture file");
        let reports = self.server_linter.run_single(&uri, Some(content)).unwrap();
        let snapshot = if reports.is_empty() {
            "No diagnostic reports".to_string()
        } else {
            reports.iter().map(get_snapshot_from_report).collect::<Vec<_>>().join("\n")
        };

        let snapshot_name = relative_file_path.replace('/', "_");
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        if let Some(suffix) = self.snapshot_suffix {
            settings.set_snapshot_suffix(suffix);
        }
        settings.bind(|| {
            insta::assert_snapshot!(snapshot_name, snapshot);
        });
    }
}
