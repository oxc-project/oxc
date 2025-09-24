use std::{
    path::Path,
    sync::{Arc, OnceLock},
};

use oxc_linter::{
    ConfigStore, LINTABLE_EXTENSIONS, TsGoLintState, loader::LINT_PARTIAL_LOADER_EXTENSIONS,
    read_to_string,
};
use rustc_hash::FxHashSet;
use tower_lsp_server::{UriExt, lsp_types::Uri};

use crate::linter::error_with_position::{
    DiagnosticReport, message_with_position_to_lsp_diagnostic_report,
};

pub struct TsgoLinter {
    state: TsGoLintState,
}

impl TsgoLinter {
    pub fn new(root_uri: &Path, config_store: ConfigStore) -> Self {
        let state = TsGoLintState::new(root_uri, config_store);
        Self { state }
    }

    pub fn lint_file(&self, uri: &Uri, content: Option<String>) -> Option<Vec<DiagnosticReport>> {
        let path = uri.to_file_path()?;

        if !Self::should_lint_path(&path) {
            return None;
        }

        let source_text = content.or_else(|| read_to_string(&path).ok())?;

        let messages = self.state.lint_source(&Arc::from(path.as_os_str()), source_text).ok()?;

        Some(
            messages
                .iter()
                .map(|e| message_with_position_to_lsp_diagnostic_report(e, uri))
                .collect(),
        )
    }

    fn should_lint_path(path: &Path) -> bool {
        static WANTED_EXTENSIONS: OnceLock<FxHashSet<&'static str>> = OnceLock::new();
        let wanted_exts = WANTED_EXTENSIONS.get_or_init(|| {
            LINTABLE_EXTENSIONS
                .iter()
                .filter(|ext| !LINT_PARTIAL_LOADER_EXTENSIONS.contains(ext))
                .copied()
                .collect()
        });

        path.extension()
            .and_then(std::ffi::OsStr::to_str)
            .is_some_and(|ext| wanted_exts.contains(ext))
    }
}
