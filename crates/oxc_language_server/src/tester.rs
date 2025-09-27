use std::{fmt::Write, path::PathBuf};

use tower_lsp_server::{
    UriExt,
    lsp_types::{CodeDescription, NumberOrString, Uri},
};

use crate::{
    linter::{
        options::{LintOptions, Run},
        server_linter::ServerLinterRun,
    },
    options::Options,
    worker::WorkspaceWorker,
};

use super::linter::error_with_position::DiagnosticReport;

/// Given a file path relative to the crate root directory, return the absolute path of the file.
pub fn get_file_path(relative_file_path: &str) -> PathBuf {
    std::env::current_dir().expect("could not get current dir").join(relative_file_path)
}

/// Given a file path relative to the crate root directory, return the URI of the file.
pub fn get_file_uri(relative_file_path: &str) -> Uri {
    Uri::from_file_path(get_file_path(relative_file_path))
        .expect("failed to convert file path to URL")
}

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
                    write!(result, "related_information[{}].message: {:?}", i, info.message)
                        .unwrap();
                    // replace everything between `file://` and `oxc_language_server` with `<variable>`, to avoid
                    // the absolute path causing snapshot test failures in different environments
                    let mut location = info.location.uri.to_string();
                    let start =
                        location.find("file://").expect("file:// protocol not found in URI");
                    let end = location
                        .find("oxc_language_server")
                        .expect("oxc_language_server not found in URI");
                    location.replace_range(
                        start + "file://".len()..end + "oxc_language_server".len(),
                        "<variable>",
                    );

                    write!(result, "\nrelated_information[{i}].location.uri: {location:?}",)
                        .unwrap();
                    write!(
                        result,
                        "\nrelated_information[{}].location.range: {:?}",
                        i, info.location.range
                    )
                    .unwrap();
                    result
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
        None => "related_information: None".to_string(),
    };
    let severity = report.diagnostic.severity;
    let source = &report.diagnostic.source;
    let tags = &report.diagnostic.tags;
    let fixed = &report.fixed_content;

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
fixed: {fixed:#?}
"
    )
}

/// Testing struct for the [linter server][crate::linter::server_linter::ServerLinter].
pub struct Tester<'t> {
    relative_root_dir: &'t str,
    options: Option<LintOptions>,
}

impl Tester<'_> {
    pub fn new(relative_root_dir: &'static str, options: Option<LintOptions>) -> Self {
        Self { relative_root_dir, options }
    }

    async fn create_workspace_worker(&self) -> WorkspaceWorker {
        let absolute_path = std::env::current_dir()
            .expect("could not get current dir")
            .join(self.relative_root_dir);
        let uri = Uri::from_file_path(absolute_path).expect("could not convert current dir to uri");
        let worker = WorkspaceWorker::new(uri);
        let option =
            &Options { lint: self.options.clone().unwrap_or_default(), ..Default::default() };
        worker.start_worker(option).await;

        worker
    }

    /// Given a relative file path (relative to `oxc_language_server` crate root), run the linter
    /// and return the resulting diagnostics in a custom snapshot format.
    pub fn test_and_snapshot_single_file(&self, relative_file_path: &str) {
        self.test_and_snapshot_single_file_with_run_type(
            relative_file_path,
            self.options.as_ref().map_or(Run::default(), |o| o.run),
        );
    }

    pub fn test_and_snapshot_multiple_file(&self, relative_file_paths: &[&str]) {
        self.test_and_snapshot_multiple_file_with_run_type(
            relative_file_paths,
            self.options.as_ref().map_or(Run::default(), |o| o.run),
        );
    }

    pub fn test_and_snapshot_single_file_with_run_type(
        &self,
        relative_file_path: &str,
        run_type: Run,
    ) {
        self.test_and_snapshot_multiple_file_with_run_type(&[relative_file_path], run_type);
    }

    #[expect(clippy::disallowed_methods)]
    pub fn test_and_snapshot_multiple_file_with_run_type(
        &self,
        relative_file_paths: &[&str],
        run_type: Run,
    ) {
        let mut snapshot_result = String::new();
        for relative_file_path in relative_file_paths {
            let uri = get_file_uri(&format!("{}/{}", self.relative_root_dir, relative_file_path));
            let reports = tokio::runtime::Runtime::new().unwrap().block_on(async {
                self.create_workspace_worker()
                    .await
                    .lint_file(
                        &uri,
                        None,
                        match run_type {
                            Run::OnSave => ServerLinterRun::OnSave,
                            Run::OnType => ServerLinterRun::OnType,
                        },
                    )
                    .await
            });

            let snapshot = if let Some(reports) = reports {
                if reports.is_empty() {
                    "No diagnostic reports".to_string()
                } else {
                    reports.iter().map(get_snapshot_from_report).collect::<Vec<_>>().join("\n")
                }
            } else {
                "File is ignored".to_string()
            };

            let _ = write!(
                snapshot_result,
                "########## \nfile: {}/{relative_file_path}\n----------\n{snapshot}\n",
                self.relative_root_dir
            );
        }

        let snapshot_name = self.relative_root_dir.replace('/', "_");
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.set_snapshot_suffix(relative_file_paths.join("_").replace('\\', "/"));
        settings.bind(|| {
            insta::assert_snapshot!(snapshot_name, snapshot_result);
        });
    }
}
