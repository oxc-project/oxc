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

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::*;
    use crate::linter::tester::Tester;
    use oxc_linter::{LintFilter, LintFilterKind, Oxlintrc};

    #[test]
    fn test_no_errors() {
        Tester::new()
            .with_snapshot_suffix("no_errors")
            .test_and_snapshot_single_file("fixtures/linter/hello_world.js");
    }

    #[test]
    fn test_no_console() {
        let config_store = ConfigStoreBuilder::default()
            .with_filter(LintFilter::deny(LintFilterKind::parse("no-console".into()).unwrap()))
            .build()
            .unwrap();
        let linter = Linter::new(LintOptions::default(), config_store).with_fix(FixKind::SafeFix);

        Tester::new_with_linter(linter)
            .with_snapshot_suffix("deny_no_console")
            .test_and_snapshot_single_file("fixtures/linter/hello_world.js");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9958
    #[test]
    fn test_issue_9958() {
        let config_store = ConfigStoreBuilder::from_oxlintrc(
            true,
            Oxlintrc::from_file(&PathBuf::from("fixtures/linter/issue_9958/.oxlintrc.json"))
                .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
        let linter = Linter::new(LintOptions::default(), config_store).with_fix(FixKind::SafeFix);

        Tester::new_with_linter(linter)
            .test_and_snapshot_single_file("fixtures/linter/issue_9958/issue.ts");
    }

    // Test case for https://github.com/oxc-project/oxc/issues/9957
    #[test]
    fn test_regexp() {
        let config_store = ConfigStoreBuilder::from_oxlintrc(
            true,
            Oxlintrc::from_file(&PathBuf::from("fixtures/linter/regexp_feature/.oxlintrc.json"))
                .unwrap(),
        )
        .unwrap()
        .build()
        .unwrap();
        let linter = Linter::new(LintOptions::default(), config_store).with_fix(FixKind::SafeFix);

        Tester::new_with_linter(linter)
            .test_and_snapshot_single_file("fixtures/linter/regexp_feature/index.ts");
    }
}
