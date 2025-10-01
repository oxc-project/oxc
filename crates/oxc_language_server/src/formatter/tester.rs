use std::{fmt::Write, path::PathBuf};

use tower_lsp_server::{
    UriExt,
    lsp_types::{TextEdit, Uri},
};

use crate::{formatter::options::FormatOptions, options::Options, worker::WorkspaceWorker};

/// Given a file path relative to the crate root directory, return the absolute path of the file.
pub fn get_file_path(relative_file_path: &str) -> PathBuf {
    std::env::current_dir().expect("could not get current dir").join(relative_file_path)
}

/// Given a file path relative to the crate root directory, return the URI of the file.
pub fn get_file_uri(relative_file_path: &str) -> Uri {
    Uri::from_file_path(get_file_path(relative_file_path))
        .expect("failed to convert file path to URL")
}

fn get_snapshot_from_text_edits(edits: &[TextEdit]) -> String {
    if edits.len() == 1 {
        // Single edit - show range and the actual formatted content with proper indentation
        let edit = &edits[0];
        let indent = " ".repeat(edit.range.start.character as usize);
        let indented_content = format!("{}{}", indent, edit.new_text);

        format!("Range: {:#?}\n\n{}", edit.range, indented_content)
    } else {
        // Multiple edits - show each edit separately
        edits
            .iter()
            .enumerate()
            .map(|(i, edit)| {
                let indent = " ".repeat(edit.range.start.character as usize);
                let indented_content = format!("{}{}", indent, edit.new_text);

                format!("Edit {}: Range: {:#?}\n{}", i + 1, edit.range, indented_content)
            })
            .collect::<Vec<_>>()
            .join("\n----------\n")
    }
}

/// Testing struct for the [formatter server][crate::formatter::server_formatter::ServerFormatter].
pub struct Tester<'t> {
    relative_root_dir: &'t str,
    options: Option<FormatOptions>,
}

impl Tester<'_> {
    pub fn new(relative_root_dir: &'static str, options: Option<FormatOptions>) -> Self {
        Self { relative_root_dir, options }
    }

    async fn create_workspace_worker(&self) -> WorkspaceWorker {
        let absolute_path = std::env::current_dir()
            .expect("could not get current dir")
            .join(self.relative_root_dir);
        let uri = Uri::from_file_path(absolute_path).expect("could not convert current dir to uri");
        let worker = WorkspaceWorker::new(uri);
        let option =
            &Options { format: self.options.clone().unwrap_or_default(), ..Default::default() };
        worker.start_worker(option).await;

        worker
    }

    pub fn format_and_snapshot_single_file(&self, relative_file_path: &str) {
        self.format_and_snapshot_multiple_file(&[relative_file_path]);
    }

    #[expect(clippy::disallowed_methods)]
    pub fn format_and_snapshot_multiple_file(&self, relative_file_paths: &[&str]) {
        let mut snapshot_result = String::new();
        for relative_file_path in relative_file_paths {
            let uri = get_file_uri(&format!("{}/{}", self.relative_root_dir, relative_file_path));
            let formatted = tokio::runtime::Runtime::new().unwrap().block_on(async {
                self.create_workspace_worker().await.format_file(&uri, None).await
            });

            let snapshot = if let Some(formatted) = formatted {
                get_snapshot_from_text_edits(&formatted)
            } else {
                "File is ignored".to_string()
            };

            let _ = write!(
                snapshot_result,
                "========================================\nFile: {}/{}\n========================================\n{}\n",
                self.relative_root_dir, relative_file_path, snapshot
            );
        }

        let snapshot_name = self.relative_root_dir.replace('/', "_");
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(
            std::env::current_dir().expect("could not get current dir").join("src/snapshots"),
        );
        settings.set_prepend_module_to_snapshot(false);
        settings.set_omit_expression(true);
        settings.set_snapshot_suffix(relative_file_paths.join("_").replace('\\', "/"));
        settings.bind(|| {
            insta::assert_snapshot!(snapshot_name, snapshot_result);
        });
    }
}
