use std::sync::Arc;

use tower_lsp::lsp_types::Url;

use oxc_linter::{FixKind, Linter};

use crate::linter::isolated_lint_handler::IsolatedLintHandler;
use crate::linter::error_with_position::DiagnosticReport;

pub struct ServerLinter {
    linter: Arc<Linter>,
}

impl ServerLinter {
    pub fn new() -> Self {
        let linter = Linter::default().with_fix(FixKind::SafeFix);
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