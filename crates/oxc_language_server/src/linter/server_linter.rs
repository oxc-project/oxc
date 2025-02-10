use std::sync::Arc;

use tower_lsp::lsp_types::Url;

use oxc_linter::{ConfigStoreBuilder, FixKind, LintOptions, Linter};

use crate::linter::error_with_position::DiagnosticReport;
use crate::linter::isolated_lint_handler::IsolatedLintHandler;

pub struct ServerLinter {
    linter: Arc<Linter>,
}

impl ServerLinter {
    pub fn new() -> Self {
        let config_store =
            ConfigStoreBuilder::default().build().expect("Failed to build config store");
        let linter = Linter::new(LintOptions::default(), config_store).with_fix(FixKind::SafeFix);
        Self { linter: Arc::new(linter) }
    }

    pub fn new_with_linter(linter: Linter) -> Self {
        Self { linter: Arc::new(linter) }
    }

    pub fn run_single(&self, uri: &Url, content: Option<String>) -> Option<Vec<DiagnosticReport>> {
        IsolatedLintHandler::new(Arc::clone(&self.linter))
            .run_single(&uri.to_file_path().unwrap(), content)
    }
}
