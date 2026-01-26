use std::{fmt::Write, path::PathBuf};

use oxc_language_server::{DiagnosticResult, Tool, ToolRestartChanges};
use tower_lsp_server::ls_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, CodeDescription, Diagnostic, NumberOrString,
    Position, Range, Uri,
};

use crate::lsp::server_linter::{ServerLinter, ServerLinterBuilder};

/// Given a file path relative to the crate root directory, return the absolute path of the file.
pub fn get_file_path(relative_file_path: &str) -> PathBuf {
    std::env::current_dir().expect("could not get current dir").join(relative_file_path)
}

/// Given a file path relative to the crate root directory, return the URI of the file.
pub fn get_file_uri(relative_file_path: &str) -> Uri {
    Uri::from_file_path(get_file_path(relative_file_path))
        .expect("failed to convert file path to URL")
}

fn get_snapshot_from_diagnostic_result(diagnostic_result: &[(Uri, Vec<Diagnostic>)]) -> String {
    if diagnostic_result.is_empty() {
        return "No diagnostics / Files are ignored".to_string();
    }

    diagnostic_result
        .iter()
        .map(|(uri, diagnostics)| {
            let mut result = String::new();
            let _ = writeln!(result, "File URI: {}", get_snapshot_safe_uri(uri));
            for diagnostic in diagnostics {
                let _ = writeln!(result, "{}", get_snapshot_from_diagnostic(diagnostic));
            }
            result
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn get_snapshot_safe_uri(uri: &Uri) -> String {
    let mut safe_uri = uri.to_string();
    let start = safe_uri.find("file://").expect("file:// protocol not found in URI");
    let end = safe_uri.find("oxlint").expect("oxlint not found in URI");
    safe_uri.replace_range(start + "file://".len()..end + "oxlint".len(), "<variable>");
    safe_uri
}

fn get_snapshot_from_diagnostic(diagnostic: &Diagnostic) -> String {
    let code = match &diagnostic.code {
        Some(NumberOrString::Number(code)) => code.to_string(),
        Some(NumberOrString::String(code)) => code.clone(),
        None => "None".to_string(),
    };
    let code_description_href = match &diagnostic.code_description {
        Some(CodeDescription { href }) => href.to_string(),
        None => "None".to_string(),
    };
    let message = diagnostic.message.clone();
    let range = diagnostic.range;
    let related_information = match &diagnostic.related_information {
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
                    let location = get_snapshot_safe_uri(&info.location.uri);

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
    let severity = diagnostic.severity;
    let source = &diagnostic.source;
    let tags = &diagnostic.tags;

    format!(
        r"
code: {code:?}
code_description.href: {code_description_href:?}
message: {message:?}
range: {range:?}
{related_information}
severity: {severity:?}
source: {source:?}
tags: {tags:?}"
    )
}

fn get_snapshot_for_code_action(code_action: &CodeAction) -> String {
    let Some(edits) = &code_action.edit else {
        return "None Workspace edits".to_string();
    };

    let Some(changes) = &edits.changes else {
        return "No changes in workspace edit".to_string();
    };

    let mut result = String::new();
    let _ = writeln!(result, "Title: {}", code_action.title);
    let _ = writeln!(result, "Is Preferred: {:?}", code_action.is_preferred);
    let _ = writeln!(
        result,
        "{}",
        changes
            .values()
            .map(|text_edits| {
                let mut result = String::new();
                for text_edit in text_edits {
                    let _ = writeln!(result, "TextEdit: {text_edit:#?}");
                }
                result
            })
            .collect::<Vec<_>>()
            .join("\n")
    );

    result
}

fn get_snapshot_from_code_action_or_command(action_or_command: &CodeActionOrCommand) -> String {
    match action_or_command {
        CodeActionOrCommand::Command(command) => format!("Command: {command:#?}"),
        CodeActionOrCommand::CodeAction(code_action) => {
            format!("CodeAction: \n{}", get_snapshot_for_code_action(code_action))
        }
    }
}

fn get_snapshot_from_report(report: &FileResult) -> String {
    let diagnostics = match &report.diagnostic {
        Err(err) => return format!("Error running diagnostics: {err}"),
        Ok(diagnostics) => diagnostics,
    };

    if diagnostics.is_empty() {
        return "No diagnostics / Files are ignored".to_string();
    }

    format!(
        "########## Diagnostic Reports
{}
########### Code Actions/Commands
{}
########### Fix All Action
{}",
        get_snapshot_from_diagnostic_result(diagnostics),
        report
            .actions
            .iter()
            .map(get_snapshot_from_code_action_or_command)
            .collect::<Vec<_>>()
            .join("\n"),
        report
            .fix_all_action
            .as_ref()
            .map_or_else(|| "None".to_string(), get_snapshot_from_code_action_or_command)
    )
}

/// Testing struct for the [linter server][crate::linter::server_linter::ServerLinter].
pub struct Tester<'t> {
    relative_root_dir: &'t str,
    options: serde_json::Value,
    builder: Option<ServerLinterBuilder>,
}

struct FileResult {
    diagnostic: DiagnosticResult,
    actions: Vec<CodeActionOrCommand>,
    fix_all_action: Option<CodeActionOrCommand>,
}

impl Tester<'_> {
    pub fn new(relative_root_dir: &'static str, options: serde_json::Value) -> Self {
        Self { relative_root_dir, options, builder: None }
    }

    pub fn with_builder(mut self, builder: ServerLinterBuilder) -> Self {
        self.builder = Some(builder);
        self
    }

    fn create_linter(&self) -> ServerLinter {
        match &self.builder {
            Some(builder) => {
                builder.build(&Self::get_root_uri(self.relative_root_dir), self.options.clone())
            }
            None => ServerLinterBuilder::default()
                .build(&Self::get_root_uri(self.relative_root_dir), self.options.clone()),
        }
    }

    pub fn get_root_uri(relative_root_dir: &str) -> Uri {
        let absolute_path =
            std::env::current_dir().expect("could not get current dir").join(relative_root_dir);

        Uri::from_file_path(absolute_path).expect("could not convert current dir to uri")
    }

    /// Given a relative file path (relative to `oxc_language_server` crate root), run the linter
    /// and return the resulting diagnostics in a custom snapshot format.
    pub fn test_and_snapshot_single_file(&self, relative_file_path: &str) {
        self.test_and_snapshot_multiple_file(&[relative_file_path]);
    }

    pub fn test_and_snapshot_multiple_file(&self, relative_file_paths: &[&str]) {
        let mut snapshot_result = String::new();
        for relative_file_path in relative_file_paths {
            let uri = get_file_uri(&format!("{}/{}", self.relative_root_dir, relative_file_path));
            let linter = self.create_linter();
            let reports = FileResult {
                diagnostic: linter.run_diagnostic(&uri, None),
                actions: linter.get_code_actions_or_commands(
                    &uri,
                    &Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX)),
                    None,
                ),
                fix_all_action: linter
                    .get_code_actions_or_commands(
                        &uri,
                        &Range::new(Position::new(0, 0), Position::new(u32::MAX, u32::MAX)),
                        Some(&vec![CodeActionKind::SOURCE_FIX_ALL]),
                    )
                    .into_iter()
                    .next(),
            };

            let _ = write!(
                snapshot_result,
                "########## \nLinted file: {}/{relative_file_path}\n----------\n{}\n",
                self.relative_root_dir,
                get_snapshot_from_report(&reports)
            );
        }

        #[expect(clippy::disallowed_methods)]
        let snapshot_name = self.relative_root_dir.replace('/', "_");
        let mut settings = insta::Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        #[expect(clippy::disallowed_methods)]
        settings.set_snapshot_suffix(relative_file_paths.join("_").replace('\\', "/"));
        settings.bind(|| {
            insta::assert_snapshot!(snapshot_name, snapshot_result);
        });
    }

    pub fn get_watcher_patterns(&self) -> Vec<String> {
        self.create_linter().get_watcher_patterns(self.options.clone())
    }

    pub fn handle_configuration_change(
        &self,
        new_options: serde_json::Value,
    ) -> ToolRestartChanges {
        let builder = ServerLinterBuilder::default();
        self.create_linter().handle_configuration_change(
            &builder,
            &Self::get_root_uri(self.relative_root_dir),
            &self.options,
            new_options,
        )
    }
}
