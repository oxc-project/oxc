use std::{
    path::Path,
    sync::{Arc, OnceLock},
};

use oxc_data_structures::rope::Rope;
use oxc_linter::{
    ConfigStore, LINTABLE_EXTENSIONS, TsGoLintState, loader::LINT_PARTIAL_LOADER_EXTENSIONS,
    read_to_string,
};
use rustc_hash::FxHashSet;
use tower_lsp_server::{UriExt, lsp_types::Uri};

use crate::linter::error_with_position::{
    DiagnosticReport, generate_inverted_diagnostics, message_to_lsp_diagnostic,
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
        let rope = Rope::from_str(&source_text);

        // TODO: Avoid cloning the source text
        let messages =
            self.state.lint_source(&Arc::from(path.as_os_str()), source_text.clone()).ok()?;

        let mut diagnostics: Vec<DiagnosticReport> = messages
            .iter()
            .map(|e| message_to_lsp_diagnostic(e, uri, &source_text, &rope))
            .collect();

        let mut inverted_diagnostics = generate_inverted_diagnostics(&diagnostics, uri);
        diagnostics.append(&mut inverted_diagnostics);

        Some(diagnostics)
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
